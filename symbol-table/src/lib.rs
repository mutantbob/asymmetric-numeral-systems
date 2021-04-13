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
    decode: Vec<(u8, u32)>,
    verbose: bool,
}

impl ANSTableUniform {
    pub fn new(freqs: SymbolFrequencies) -> ANSTableUniform {
        let frequencies = freqs.frequencies;
        let sum_frequencies = frequencies.iter().sum();
        //println!("sum_frequencies = {}", sum_frequencies);

        let mut transforms: Vec<Vec<u32>> = (0..256).map(|_| Vec::new()).collect();
        let mut backward: Vec<(u8, u32)> = Vec::new();

        let mut accum = [0; 256];

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
            if a != 0 {
                println!("unexpected accum[{}] == {}", i, a);
            }
        }

        ANSTableUniform {
            frequencies,
            sum_frequencies,
            encode: transforms,
            decode: backward,
            verbose: false,
        }
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
