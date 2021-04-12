extern crate symbol_table;

use std::fs::File;
use std::io;
use std::io::{Error, Write};
use symbol_table::{ANSTableUniform, SymbolFrequencies};

pub struct StreamingANSUniform {
    pub table: ANSTableUniform,
    pub underflow_bits: u8,
    pub verbose: bool,
}

impl StreamingANSUniform {
    pub fn new(freqs: SymbolFrequencies, underflow_bits: u8) -> StreamingANSUniform {
        let table = ANSTableUniform::new(freqs);
        StreamingANSUniform {
            table,
            underflow_bits,
            verbose: false,
        }
    }

    pub fn encode(&self, message: &[u8]) -> Vec<u8> {
        let mut x = 0;

        let mut rval: Vec<u8> = Vec::new();
        //for symbol in message.iter().reverse() {
        for idx in (0..message.len()).rev() {
            let symbol = message[idx];
            let mut new_x = self.table.append_encode64(x, symbol);
            if new_x >> (self.underflow_bits + 8) != 0 {
                if self.verbose {
                    println!("{:x}.{} overflows to {:x}", x, symbol as char, new_x)
                }
                rval.push(x as u8);
                x >>= 8;
                new_x = self.table.append_encode64(x, symbol);
            }
            if self.verbose {
                println!("{:x}.{} becomes {:x}", x, symbol as char, new_x);
            }
            x = new_x;
        }

        while x != 0 {
            if self.verbose {
                println!("push LSB of {:x} to stream", x);
            }
            rval.push((x & 0xff) as u8);
            x >>= 8;
        }

        rval
    }

    pub fn bad_encode1(&self, message: &[u8]) -> Vec<u8> {
        let mut x = 0;

        let mut rval: Vec<u8> = Vec::new();
        //for symbol in message.iter().reverse() {
        for idx in (0..message.len()).rev() {
            let symbol = message[idx];
            let old_x = x;
            x = self.table.append_encode64(x, symbol);
            if self.verbose {
                println!("{:x}.{} becomes {:x}", old_x, symbol as char, x);
            }
            if x >> 16 != 0 {
                if self.verbose {
                    println!("push LSB of {:x} to stream", x);
                }
                rval.push((x & 0xff) as u8);
                x >>= 8;
            }
        }

        while x != 0 {
            if self.verbose {
                println!("push LSB of {:x} to stream", x);
            }
            rval.push((x & 0xff) as u8);
            x >>= 8;
        }

        rval
    }

    pub fn decode(&self, stream: &[u8]) -> Vec<u8> {
        //let mut cursor: i64 = (stream.len() - 1) as i64;

        let mut iter = stream.iter().rev();

        let mut rval = Vec::new();
        let mut x: u64 = 0;
        loop {
            if x >> self.underflow_bits == 0 {
                match iter.next() {
                    Some(&stream_byte) => {
                        if self.verbose {
                            println!("pulled LSB from stream {:x} {:02x}", x, stream_byte);
                        }
                        x = (x << 8) | (stream_byte as u64);
                        continue;
                    }
                    None => {
                        if x == 0 {
                            break;
                        }
                    }
                }
            }

            let (symbol, new_x) = self.table.decode64(x);
            if self.verbose {
                println!("{:x} becomes {:x}.'{}'", x, new_x, symbol as char);
            }
            rval.push(symbol);
            x = new_x;
        }

        rval
    }

    pub fn bad_decode1(&self, stream: &[u8]) -> Vec<u8> {
        //let mut cursor: i64 = (stream.len() - 1) as i64;

        let mut iter = stream.iter().rev();

        let mut rval = Vec::new();
        let mut x: u64 = 0;
        loop {
            let (mut symbol, mut new_x) = self.table.decode64(x);
            if new_x >> 8 == 0 {
                match iter.next() {
                    Some(&stream_byte) => {
                        if self.verbose {
                            println!("pulled LSB from stream {:x} {:02x}", x, stream_byte);
                        }
                        x = (x << 8) | (stream_byte as u64);
                        if new_x >> 8 == 0 {
                            continue;
                        } else {
                            let tmp = self.table.decode64(x);
                            symbol = tmp.0;
                            new_x = tmp.1;
                        }
                    }
                    None => {
                        if x == 0 {
                            break;
                        }
                    }
                }
            }

            //let (symbol, new_x) = self.table.decode64(x);
            if self.verbose {
                println!("{:x} becomes {:x}.'{}'", x, new_x, symbol as char);
            }
            rval.push(symbol);
            x = new_x;
        }

        rval
    }
}

fn main() -> Result<(), Error> {
    {
        let fname = "../test-data/out/atmm.bin";
        println!("symbol frequency file {}", fname);
        demonstration1(fname, b"what is a man, but a miserable pile of secrets?", 8)?;
        demonstration1(
            fname,
            b"what is a man, but a miserable pile of secrets?",
            16,
        )?;
        demonstration1(
            fname,
            b"what is a man, but a miserable pile of secrets?",
            32,
        )?;
    }

    Ok(())
}

fn demonstration1(symbol_fname: &str, message: &[u8], underflow_bits: u8) -> Result<(), Error> {
    let mut symbol_file = File::open(symbol_fname)?;
    let symbols = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

    let symbols = scale_frequencies(16, &symbols, false);

    let mut uans = StreamingANSUniform::new(symbols, underflow_bits);
    uans.verbose = false;

    let compressed = uans.encode(message);

    println!(
        "compressed to {} bytes (UB={})",
        compressed.len(),
        underflow_bits
    );

    let uncompressed = uans.decode(&compressed);

    io::stdout().write_all(&uncompressed)?;
    println!();

    assert!(uncompressed == message, "encode/decode mismatch");
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
