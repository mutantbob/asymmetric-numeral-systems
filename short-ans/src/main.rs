extern crate symbol_table;

use std::env;
use std::fs::File;
use std::io::Error;
use symbol_table::SymbolFrequencies;

struct ANSTable {
    frequencies: [u32; 256],
    sum_frequencies: u32,
    encode: Vec<Vec<u32>>,
    decode: Vec<(u8, u32)>,
}

impl ANSTable {
    pub fn new(freqs: SymbolFrequencies) -> ANSTable {
        let frequencies = freqs.frequencies;
        let sum_frequencies = //freqs.frequencies.iter().fold(0, |a,b| a+b);
        frequencies.iter().sum();

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

        println!("{} better == {}", cursor, sum_frequencies);
        for (i, &a) in accum.iter().enumerate() {
            if a != 0 {
                println!("unexpected accum[{}] == {}", i, a);
            }
        }

        ANSTable {
            frequencies,
            sum_frequencies,
            encode: transforms,
            decode: backward,
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
        println!(
            "{} = {}*{} + {} ;  rval = {}*{} + {} = 0x{:x}",
            val, freq, cycle, phase, self.sum_frequencies, cycle, encoded, rval
        );
        rval
    }

    pub fn append_encode64(&self, val: u64, symbol: u8) -> u64 {
        let freq = self.frequencies[symbol as usize];
        let cycle = val / (freq as u64);
        let phase = val % (freq as u64);
        let encoded = self.encode[symbol as usize][phase as usize];
        let rval = cycle * (self.sum_frequencies as u64) + (encoded as u64);
        println!(
            "{} = {}*{} + {} ;  rval = {}*{} + {} = 0x{:x}",
            val, freq, cycle, phase, self.sum_frequencies, cycle, encoded, rval
        );
        rval
    }

    pub fn decode32(&self, val: u32) -> (u8, u32) {
        let cycle = val / self.sum_frequencies;
        let phase = val % self.sum_frequencies;

        let (symbol, tmp) = self.decode[phase as usize];
        let rval = cycle * self.frequencies[symbol as usize] + tmp;
        println!(
            "{} = {}*{} + {}; rval = {}*{} + {} = 0x{:x}",
            val,
            self.sum_frequencies,
            cycle,
            phase,
            rval,
            self.frequencies[symbol as usize],
            tmp,
            rval
        );
        (symbol, rval)
    }
    pub fn decode64(&self, val: u64) -> (u8, u64) {
        let sum_frequencies = self.sum_frequencies as u64;
        let cycle = val / sum_frequencies;
        let phase = val % sum_frequencies;

        let (symbol, tmp) = self.decode[phase as usize];
        let sym_freq = self.frequencies[symbol as usize];
        let rval = cycle * (sym_freq as u64) + (tmp as u64);
        println!(
            "{} = {}*{} + {}; rval = {}*{} + {} = 0x{:x}",
            val, sum_frequencies, cycle, phase, rval, sym_freq, tmp, rval
        );
        (symbol, rval)
    }
}

//

fn main() -> Result<(), Error> {
    let args = env::args();
    let mut args = args.skip(1);

    let fname = args.next().unwrap();
    println!("symbol frequency file {}", &fname);
    let mut symbol_file = File::open(fname)?;
    let frequencies = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

    let ans_table = ANSTable::new(frequencies);

    let x = ans_table.append_encode(0, b'R');
    let x = ans_table.append_encode(x, b'o');
    let x = ans_table.append_encode(x, b'b');
    let x = ans_table.append_encode(x, b'e');
    let x = ans_table.append_encode(x, b'r');
    let x = ans_table.append_encode64(x as u64, b't');

    println!("x={}", x);

    let mut out: String = Default::default();
    let (symbol, x) = ans_table.decode64(x);
    out.insert(0, symbol as char);
    let (symbol, x) = ans_table.decode32(x as u32);
    out.insert(0, symbol as char);
    let (symbol, x) = ans_table.decode32(x as u32);
    out.insert(0, symbol as char);
    let (symbol, x) = ans_table.decode32(x as u32);
    out.insert(0, symbol as char);
    let (symbol, x) = ans_table.decode32(x as u32);
    out.insert(0, symbol as char);
    let (symbol, x) = ans_table.decode32(x as u32);
    out.insert(0, symbol as char);

    println!("final val = {} (should be 0)", x);
    println!("reconstructed : {}", out);
    Ok(())
}
