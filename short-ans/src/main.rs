extern crate symbol_table;

use std::env;
use std::fmt::{Display, LowerHex};
use std::fs::File;
use std::io::Error;
use symbol_table::SymbolFrequencies;

struct ANSTableUniform {
    frequencies: [u32; 256],
    sum_frequencies: u32,
    encode: Vec<Vec<u32>>,
    decode: Vec<(u8, u32)>,
    verbose: bool,
}

impl ANSTableUniform {
    pub fn new(freqs: SymbolFrequencies) -> ANSTableUniform {
        let frequencies = freqs.frequencies;
        let sum_frequencies = frequencies.iter().sum();
        println!("sum_frequencies = {}", sum_frequencies);

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

        assert!(cursor == sum_frequencies, "malfunction building symbol table: (cursor ={}) != (sum_frequencies={})", cursor, sum_frequencies);

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
        let cycle = val / (freq as u64);
        let phase = val % (freq as u64);
        let encoded = self.encode[symbol as usize][phase as usize];
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

fn main() -> Result<(), Error> {
    let args = env::args();
    let mut args = args.skip(1);

    {
        let fname = args.next().unwrap();
        println!("symbol frequency file {}", &fname);
        demonstration1(&fname, b"Robert")?;
    }

    {
        let fname = "../test-data/out/mpeg.bin";
        println!("symbol frequency file {}", fname);
        demonstration1(fname, b"Robert")?;
    }

    {
        let fname = "../test-data/out/atmm.bin";
        println!("symbol frequency file {}", fname);
        demonstration1(fname, b"Robert")?;
    }

    Ok(())
}

fn demonstration1(fname: &str, test_data: &[u8]) -> Result<(), Error> {
    let mut symbol_file = File::open(fname)?;
    let frequencies = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

    let mut ans_table = ANSTableUniform::new(frequencies);
    //ans_table.verbose = true;

    let x = simple_encode(&ans_table, test_data);

    let bit_count = count_bits(x);

    println!("x={}\trequires {} bits", x, bit_count);

    let mut out: String = Default::default();
    for symbol in simple_decode(&ans_table, x) {
        out.push(symbol as char);
    }

    println!("reconstructed : {}", out);

    Ok(())
}

fn simple_encode(ans_table: &ANSTableUniform, symbols: &[u8]) -> u64 {
    let mut x = 0;
    for &symbol in symbols {
        x = ans_table.append_encode64(x, symbol);
    }
    x
}

fn simple_decode(ans_table: &ANSTableUniform, mut val: u64) -> Vec<u8> {
    let mut rval = Vec::new();
    while val > 0 {
        let (symbol, new_val) = ans_table.decode64(val);
        rval.insert(0, symbol);
        val = new_val;
    }
    rval
}

fn count_bits(mut val: u64) -> u32 {
    let mut rval = 0;
    while val != 0 {
        rval += 1;
        val >>= 1;
    }
    rval
}
