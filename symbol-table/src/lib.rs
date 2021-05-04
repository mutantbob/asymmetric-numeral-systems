extern crate byteorder;

use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use std::fmt::{Display, LowerHex};
use std::io::{Error, Read};

pub struct SymbolFrequencies {
    pub frequencies: [u32; 256],
}

impl SymbolFrequencies {
    pub fn new() -> SymbolFrequencies {
        SymbolFrequencies {
            frequencies: [0; 256],
        }
    }

    pub fn scan_file(&mut self, f: &mut dyn Read) -> Result<(), Error> {
        let mut buffer = [0; 4 << 10];

        loop {
            let count = f.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            for &symbol in &buffer[..count] {
                self.frequencies[symbol as usize] += 1;
            }
        }
        Ok(())
    }

    pub fn parse_binary_symbol_table(src: &mut dyn Read) -> Result<SymbolFrequencies, Error> {
        let mut frequencies = [0; 256];
        src.read_u32_into::<BigEndian>(&mut frequencies)?;
        Ok(SymbolFrequencies { frequencies })
    }

    pub fn missing_symbols_become_one(src: &SymbolFrequencies) -> SymbolFrequencies {
        let mut new_frequencies: [u32; 256] = [0; 256];
        for (symbol, &freq) in src.frequencies.iter().enumerate() {
            new_frequencies[symbol] = if freq > 0 { freq } else { 1 };
        }
        SymbolFrequencies {
            frequencies: new_frequencies,
        }
    }
}

impl Default for SymbolFrequencies {
    fn default() -> Self {
        Self::new()
    }
}

//
//
//

pub struct ANSTableUniform {
    pub frequencies: [u32; 256],
    pub sum_frequencies: u32,
    pub encode: Vec<Vec<u32>>,
    pub decode: Vec<(u8, u32)>,
    pub verbose: bool,
}

impl ANSTableUniform {
    pub fn new(freqs: SymbolFrequencies) -> ANSTableUniform {
        let frequencies = freqs.frequencies;
        let sum_frequencies = frequencies.iter().sum();
        //println!("sum_frequencies = {}", sum_frequencies);

        let (transforms, backward) =
            ANSTableUniform::build_tables(&frequencies, sum_frequencies, sum_frequencies / 2);

        ANSTableUniform {
            frequencies,
            sum_frequencies,
            encode: transforms,
            decode: backward,
            verbose: false,
        }
    }

    pub fn build_tables(
        frequencies: &[u32; 256],
        sum_frequencies: u32,
        accum_start: u32,
    ) -> (Vec<Vec<u32>>, Vec<(u8, u32)>) {
        let mut transforms: Vec<Vec<u32>> = (0..256).map(|_| Vec::new()).collect();
        let mut backward: Vec<(u8, u32)> = Vec::new();

        let mut accum = [accum_start; 256];

        let mut cursor = 0;

        for _i in 0..sum_frequencies {
            for symbol in 0..256 {
                accum[symbol] += frequencies[symbol];
                if accum[symbol] >= sum_frequencies {
                    let decoded = transforms[symbol].len();
                    transforms[symbol].push(cursor);
                    backward.push((symbol as u8, decoded as u32));

                    cursor += 1;
                    accum[symbol] -= sum_frequencies;
                }
            }
        }

        assert!(
            cursor == sum_frequencies,
            "malfunction building symbol table: (cursor ={}) != (sum_frequencies={})",
            cursor,
            sum_frequencies
        );

        for (i, &a) in accum.iter().enumerate() {
            if a != accum_start {
                println!("unexpected accum[{}] == {}", i, a);
            }
        }
        (transforms, backward)
    }

    pub fn append_encode(&self, val: u32, symbol: u8) -> u32 {
        self.append_encode32(val, symbol)
    }

    pub fn append_encode32(&self, val: u32, symbol: u8) -> u32 {
        let freq = self.frequencies[symbol as usize];
        let cycle = val / freq;
        let phase = val % freq;
        let encoded = self.encode[symbol as usize][phase as usize];
        let rval = cycle * self.sum_frequencies + encoded;
        if self.verbose {
            ANSTableUniform::log_encode(
                val,
                freq,
                cycle,
                phase,
                encoded,
                rval,
                self.sum_frequencies,
            );
        }
        rval
    }

    fn log_encode<T: Display + LowerHex>(
        val: T,
        symbol_frequency: u32,
        cycle: T,
        phase: T,
        encoded: u32,
        rval: T,
        sum_frequencies: u32,
    ) {
        println!(
            "{} = {}*{} + {} ;  rval = {}*{} + {} = 0x{:x}",
            val, symbol_frequency, cycle, phase, sum_frequencies, cycle, encoded, rval
        );
    }

    pub fn append_encode64(&self, val: u64, symbol: u8) -> u64 {
        let freq = self.frequencies[symbol as usize];
        assert!(
            freq != 0,
            "symbol {} does not appear in symbol table",
            symbol
        );
        let cycle = val / (freq as u64);
        let phase = val % (freq as u64);
        let encoded = self.encode[symbol as usize][phase as usize];
        //println!("debug for {}@{} :\t {:x}*{}+{}", symbol, freq, cycle, self.sum_frequencies, encoded);
        let rval = cycle * (self.sum_frequencies as u64) + (encoded as u64);
        if self.verbose {
            ANSTableUniform::log_encode(
                val,
                freq,
                cycle,
                phase,
                encoded,
                rval,
                self.sum_frequencies,
            );
        }
        rval
    }

    pub fn decode32(&self, val: u32) -> (u8, u32) {
        let cycle = val / self.sum_frequencies;
        let phase = val % self.sum_frequencies;

        let (symbol, tmp) = self.decode[phase as usize];
        let sym_freq = self.frequencies[symbol as usize];
        let rval = cycle * sym_freq + tmp;
        if self.verbose {
            ANSTableUniform::log_decode(
                val,
                self.sum_frequencies,
                cycle,
                phase,
                tmp,
                sym_freq,
                rval,
            );
        }
        (symbol, rval)
    }

    pub fn decode64(&self, val: u64) -> (u8, u64) {
        let sum_frequencies = self.sum_frequencies as u64;
        let cycle = val / sum_frequencies;
        let phase = val % sum_frequencies;

        let (symbol, tmp) = self.decode[phase as usize];
        let sym_freq = self.frequencies[symbol as usize];
        let rval = cycle * (sym_freq as u64) + (tmp as u64);
        if self.verbose {
            ANSTableUniform::log_decode(val, sum_frequencies, cycle, phase, tmp, sym_freq, rval);
        }
        (symbol, rval)
    }

    fn log_decode<T: Display + LowerHex>(
        val: T,
        sum_frequencies: T,
        cycle: T,
        phase: T,
        tmp: u32,
        sym_freq: u32,
        rval: T,
    ) {
        println!(
            "{} = {}*{} + {}; rval = {}*{} + {} = 0x{:x}",
            val, sum_frequencies, cycle, phase, rval, sym_freq, tmp, rval
        );
    }
}

//
//
//

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
            if frequency == 0 {
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
        }
    }

    /// For `message_backwards` you probably want something like `message.iter().rev()`.
    ///
    /// For `initial_value` you probably want `1`, and you absolutely do not want `0`.
    pub fn encode<'a, I>(&self, message_backwards: I, initial_value: u64) -> Vec<u8>
    where
        I: Iterator<Item = &'a u8>,
    {
        if initial_value == 0 {
            panic!("initial_value for encode() must not be {}", initial_value);
        }
        let mut x = initial_value;

        let mut rval: Vec<u8> = Vec::new();
        let mut rval_sink = |byte| rval.push(byte);
        for &symbol in message_backwards {
            //println!("symbol = {}", symbol);
            let mut new_x = self.table.append_encode64(x, symbol);
            if new_x >> (self.underflow_bits + 8 * self.bytes_to_stream) != 0 {
                if self.verbose {
                    println!("{:x}.{} overflows to {:x}", x, symbol as char, new_x)
                }
                x = self.push_quantum(x, &mut rval_sink);
                new_x = self.table.append_encode64(x, symbol);
            }
            if self.verbose {
                println!("{:x}.{} becomes {:x}", x, symbol as char, new_x);
            }
            x = new_x;
        }
        //println!("exhausted bytes to encode");

        while x != 0 {
            x = self.push_quantum(x, &mut rval_sink);
        }

        rval
    }

    /// for `message_backwards` you probably want something like `message.iter().rev()`
    pub fn encode_to_sink<'a, I, E>(
        &self,
        message_backwards: I,
        sink: &mut dyn FnMut(u8) -> Result<(), E>,
        initial_value: u64,
    ) -> Result<(), E>
    where
        I: Iterator<Item = &'a u8>,
    {
        if initial_value == 0 {
            panic!("initial_value for encode() must not be {}", initial_value);
        }
        let mut x = initial_value;

        for &symbol in message_backwards {
            //println!("symbol = {}", symbol);
            let mut new_x = self.table.append_encode64(x, symbol);
            if new_x >> (self.underflow_bits + 8 * self.bytes_to_stream) != 0 {
                if self.verbose {
                    println!("{:x}.{} overflows to {:x}", x, symbol as char, new_x)
                }
                x = self.push_quantum2(x, sink)?;
                new_x = self.table.append_encode64(x, symbol);
            }
            if self.verbose {
                println!("{:x}.{} becomes {:x}", x, symbol as char, new_x);
            }
            x = new_x;
        }
        //println!("exhausted bytes to encode");

        while x != 0 {
            x = self.push_quantum2(x, sink)?;
        }

        Ok(())
    }

    fn push_quantum(&self, mut x: u64, sink: &mut dyn FnMut(u8)) -> u64 {
        for _i in 0..self.bytes_to_stream {
            if self.verbose {
                println!("push LSB of {:x} to stream", x);
            }
            sink(x as u8);
            x >>= 8;
        }
        x
    }

    fn push_quantum2<E>(
        &self,
        mut x: u64,
        sink: &mut dyn FnMut(u8) -> Result<(), E>,
    ) -> Result<u64, E> {
        for _i in 0..self.bytes_to_stream {
            if self.verbose {
                println!("push LSB of {:x} to stream", x);
            }
            sink(x as u8)?;
            x >>= 8;
        }
        Ok(x)
    }

    /// `eos_marker` is the same value passed to `encode()` as `initial_value`
    pub fn decode(&self, stream: &[u8], eos_marker: u64) -> Result<Vec<u8>, String> {
        //let mut cursor: i64 = (stream.len() - 1) as i64;
        if eos_marker == 0 {
            panic!("eos_marker for decode() must not be {}", eos_marker);
        }

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

        while x != eos_marker {
            let (symbol, new_x) = self.table.decode64(x);
            if self.verbose {
                println!("{:x} becomes {:x}.'{}'", x, new_x, symbol as char);
            }
            rval.push(symbol);
            if new_x < 1 {
                return Err("failed to reach EOS marker".to_string());
            }
            x = new_x;
        }

        Ok(rval)
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
//
//

#[cfg(test)]
mod tests {
    use crate::{StreamingANSUniform, SymbolFrequencies};

    #[test]
    fn test1() {
        let mut freqs = SymbolFrequencies::new();
        freqs.frequencies[0] = 1;
        freqs.frequencies[1] = 3;

        let ansu = StreamingANSUniform::new(freqs, 16, 2);

        let iv = 1;
        {
            let orig = vec![0, 0];
            let encoded = ansu.encode(orig.iter().rev(), iv);
            let decoded = ansu.decode(&encoded, iv).unwrap();
            assert_eq!(orig, decoded);
        }
        {
            let orig = vec![1, 1];
            let encoded = ansu.encode(orig.iter().rev(), iv);
            let decoded = ansu.decode(&encoded, iv).unwrap();
            assert_eq!(orig, decoded);
        }
    }
}
