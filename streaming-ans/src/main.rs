extern crate symbol_table;

use std::fs::File;
use std::io::{Error, Read, Write};
use std::{io, panic};
use symbol_table::{ANSTableUniform, SymbolFrequencies};

pub struct StreamingANSUniform {
    pub table: ANSTableUniform,
    pub underflow_bits: u8,
    pub bytes_to_stream: u8,
    pub verbose: bool,
}

impl StreamingANSUniform {
    pub fn new(
        freqs: SymbolFrequencies,
        underflow_bits: u8,
        bytes_to_stream: u8,
    ) -> StreamingANSUniform {
        let table = ANSTableUniform::new(freqs);
        StreamingANSUniform::panic_if_unbalanced(&table, underflow_bits, bytes_to_stream);

        StreamingANSUniform {
            table,
            underflow_bits,
            bytes_to_stream,
            verbose: false,
        }
    }

    pub fn panic_if_unbalanced(table: &ANSTableUniform, underflow_bits: u8, bytes_to_stream: u8) {
        let bits_to_stream = 8 * bytes_to_stream;
        assert!(
            underflow_bits >= bits_to_stream,
            "underflow_bits {} is too small ( < 8*{} )",
            underflow_bits,
            bytes_to_stream
        );

        let max_result_bits = underflow_bits + 2 * bits_to_stream;
        if 64 < max_result_bits {
            panic!(
                "encoding process will probably overflow ( 64 < {} + 2*{} )",
                underflow_bits, bits_to_stream
            );
        }

        let max_working_x = (1u64 << (underflow_bits + bits_to_stream)) - 1;
        for (symbol, &frequency) in table.frequencies.iter().enumerate() {
            if frequency <= 0 {
                continue;
            }
            if frequency > 0 && table.sum_frequencies > (frequency << (bits_to_stream)) {
                println!("symbol {} frequency is small enough that encoding could jump by too many bits ( {} > {} << (8*{}) )",
                         symbol, table.sum_frequencies, frequency, bytes_to_stream);
            }

            let cycle = max_working_x / (frequency as u64);
            let jump = *table.encode[symbol].last().unwrap();
            let x2 = (cycle as u128) * (table.sum_frequencies as u128) + (jump as u128);
            if x2 >> (max_result_bits) > 0 {
                panic!("symbol {} frequency is small enough that encoding could jump by too many bits {:x}.{} = {:x} >= (1<<{})",
                       symbol, max_working_x, symbol, x2, max_result_bits);
            }

            if false && 41 == symbol {
                println!(
                    "symbol={}, freq={}, max_x = 0x{:x}, cycle={}, jump={}, x2=0x{:x}",
                    symbol, frequency, max_working_x, cycle, jump, x2
                );
            }
        }
    }

    pub fn encode<'a, I>(&self, message: I) -> Vec<u8>
    where
        I: Iterator<Item = &'a u8> + DoubleEndedIterator,
    {
        let mut x = 0;

        let mut rval: Vec<u8> = Vec::new();
        for &symbol in message.rev() {
            //println!("symbol = {}", symbol);
            let mut new_x = self.table.append_encode64(x, symbol);
            if new_x >> (self.underflow_bits + 8 * self.bytes_to_stream) != 0 {
                if self.verbose {
                    println!("{:x}.{} overflows to {:x}", x, symbol as char, new_x)
                }
                x = self.push_quantum(x, &mut rval);
                new_x = self.table.append_encode64(x, symbol);
            }
            if self.verbose {
                println!("{:x}.{} becomes {:x}", x, symbol as char, new_x);
            }
            x = new_x;
        }
        //println!("exhausted bytes to encode");

        while x != 0 {
            x = self.push_quantum(x, &mut rval);
        }

        rval
    }

    fn push_quantum(&self, mut x: u64, sink: &mut Vec<u8>) -> u64 {
        for _i in 0..self.bytes_to_stream {
            if self.verbose {
                println!("push LSB of {:x} to stream", x);
            }
            sink.push(x as u8);
            x >>= 8;
        }
        x
    }

    pub fn decode(&self, stream: &[u8]) -> Vec<u8> {
        //let mut cursor: i64 = (stream.len() - 1) as i64;

        let mut iter = stream.iter().rev();

        let mut rval = Vec::new();
        let mut x: u64 = 0;

        while x >> self.underflow_bits == 0 {
            match self.read_quantum(&mut iter, x) {
                None => break,
                Some(new_x) => x = new_x,
            }
        }
        loop {
            //println!("decode state x={}", x);
            if x >> self.underflow_bits == 0 {
                match self.read_quantum(&mut iter, x) {
                    None => break,
                    Some(new_x) => x = new_x,
                }
            }

            let (symbol, new_x) = self.table.decode64(x);
            if self.verbose {
                println!("{:x} becomes {:x}.'{}'", x, new_x, symbol as char);
            }
            rval.push(symbol);
            x = new_x;
        }

        while x != 0 {
            let (symbol, new_x) = self.table.decode64(x);
            if self.verbose {
                println!("{:x} becomes {:x}.'{}'", x, new_x, symbol as char);
            }
            rval.push(symbol);
            x = new_x;
        }

        rval
    }

    fn read_quantum(&self, iter: &mut dyn Iterator<Item = &u8>, mut x: u64) -> Option<u64> {
        for i in 0..self.bytes_to_stream {
            match iter.next() {
                Some(&stream_byte) => {
                    if self.verbose {
                        println!("pulled LSB from stream {:x} {:02x}", x, stream_byte);
                    }
                    //token |= (stream_byte as u64) << (i * 8);
                    x = (x << 8) | (stream_byte as u64);
                }
                None => {
                    if i == 0 {
                        return None;
                    }
                    break;
                }
            }
        }
        /*
        //println!("x={:x};\ttoken {:x}", x, token);
        if x == 0 && token == 0 {
            if true { panic!("debug") }
            return None;
        }
        Some( (x << (8*self.bytes_to_stream)) | token)*/
        //println!("read_quantum returns {}", x);
        Some(x)
    }
}

//

struct ResultWrapper<'a> {
    message: std::slice::Iter<'a, u8>,
}

impl<'a> ResultWrapper<'a> {
    pub fn new(message: &'a [u8]) -> ResultWrapper<'a> {
        ResultWrapper {
            message: message.iter(),
        }
    }
}

impl<'a> Iterator for ResultWrapper<'a> {
    type Item = Result<u8, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.message.next().map(|&val| Ok(val))
    }
}

impl<'a> DoubleEndedIterator for ResultWrapper<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.message.next_back().map(|&val| Ok(val))
    }
}

//

fn main() -> Result<(), Error> {
    {
        let message2 = slurp("../test-data/at-the-mountains-of-madness.html")?;
        let message3 = slurp("../test-data/dream-quest.html")?;
        let message4 = slurp("../test-data/iso13818-2.pdf")?;
        for symbol_fname in &[
            "../test-data/out/atmm.bin",
            "../test-data/out/dq.bin",
            "../test-data/out/mpeg.bin",
        ] {
            println!("symbol frequency file {}", symbol_fname);

            let messages: Vec<&[u8]> = vec![
                b"what is a man, but a miserable pile of secrets?",
                &message2,
                &message3,
                &message4,
            ];
            for message in &messages {
                let result = panic::catch_unwind(|| demo_suite1(symbol_fname, message, true));
                match result {
                    Ok(result) => {
                        if let Err(e) = result {
                            println!("malfunction {:?}", e);
                        }
                    }
                    Err(e) => {
                        println!("malfunction {:?}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

fn demo_suite1(symbol_fname: &str, message: &[u8], backfill_missing_symbols: bool) -> Result<(), Error> {
    let message1 = message;
    println!("  message.len() = {}", message.len());
    //explosion1(fname)?;

    if false {
        demonstration1(symbol_fname, message1, 16, 2, backfill_missing_symbols)?;
        demonstration1(symbol_fname, message1, 32, 2, backfill_missing_symbols)?;
    }

    //
    {
        let payload = message;
        //let payload:&[u8] = &payload;
        demonstration1(symbol_fname, &payload, 16, 2, backfill_missing_symbols)?;
        demonstration1(symbol_fname, &payload, 24, 2, backfill_missing_symbols)?;
        demonstration1(symbol_fname, &payload, 32, 2, backfill_missing_symbols)?;
    }

    Ok(())
}

fn slurp(fname: &str) -> Result<Vec<u8>, Error> {
    let mut f = File::open(fname)?;
    let mut payload = Vec::new();
    let _count = f.read_to_end(&mut payload)?;
    Ok(payload)
}

fn demonstration1(
    symbol_fname: &str,
    message: &[u8],
    underflow_bits: u8,
    bytes_to_stream: u8,
    backfill_missing_symbols: bool,
) -> Result<(), Error> {
    println!("demonstration(,,{})", underflow_bits);
    let mut symbol_file = File::open(symbol_fname)?;
    let symbols = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

    let symbols = if backfill_missing_symbols {
        SymbolFrequencies::missing_symbols_become_one(&symbols)
    } else {
        symbols
    };

    let symbols = scale_frequencies(16, &symbols, false);

    let mut uans = StreamingANSUniform::new(symbols, underflow_bits, bytes_to_stream);
    uans.verbose = false;

    demonstration1a(&mut uans, message, underflow_bits)
}

fn demonstration1a(
    uans: &mut StreamingANSUniform,
    message: &[u8],
    underflow_bits: u8,
) -> Result<(), Error> {
    let compressed = uans.encode(message.iter());

    println!(
        "compressed to {} bytes (UB={})",
        compressed.len(),
        underflow_bits
    );

    let uncompressed = uans.decode(&compressed);

    if false {
        io::stdout().write_all(&uncompressed)?;
        println!();
    }

    assert!(uncompressed == message, "encode/decode mismatch");
    Ok(())
}

fn explosion1(symbol_fname: &str) -> Result<(), Error> {
    let mut symbol_file = File::open(symbol_fname)?;
    let symbols = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

    let symbols = scale_frequencies(16, &symbols, false);

    let ans = ANSTableUniform::new(symbols);

    StreamingANSUniform::panic_if_unbalanced(&ans, 40, 2);

    let x = ans.append_encode64(0x377a794fef0e04, 41);
    println!("x = {}", x);

    Ok(())
}

fn scale_frequencies(num_bits: u8, raw: &SymbolFrequencies, verbose: bool) -> SymbolFrequencies {
    let mut indices: Vec<usize> = (0..raw.frequencies.len()).collect();

    indices.sort_by(|&a, &b| raw.frequencies[a].cmp(&raw.frequencies[b]));

    let mut old_sum: u32 = raw.frequencies.iter().sum();
    let mut target_sum = 1 << num_bits;

    let mut new_frequencies = [0u32; 256];

    for symbol in indices {
        let freq = raw.frequencies[symbol];
        let mut new_freq = target_sum * freq / old_sum;
        if new_freq < 1 && freq > 0 {
            new_freq = 1;
        }
        new_frequencies[symbol] = new_freq;
        if verbose {
            println!("scaling s{}  @ {} to {}", symbol, freq, new_freq);
        }

        old_sum -= freq;
        target_sum -= new_freq;
    }

    assert!(
        1u32 << num_bits == new_frequencies.iter().sum(),
        "I donked up the frequency scaling math somehow"
    );

    SymbolFrequencies {
        frequencies: new_frequencies,
    }
}
