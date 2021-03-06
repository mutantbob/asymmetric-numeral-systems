extern crate symbol_table;

use std::env;
use std::fs::File;
use std::io::Error;
use symbol_table::{ANSTableUniform, SymbolFrequencies};

//

/// Encode a test string using different symbol tables constructed from different source documents.
/// The symbol tables are built using the `measure` app.
fn main() -> Result<(), Error> {
    let args = env::args();
    let args = args.skip(1);
    let mut symbol_tables: Vec<String> = args.collect();
    if symbol_tables.is_empty() {
        symbol_tables = [
            "../test-data/out/atmm.bin",
            "../test-data/out/dq.bin",
            "../test-data/out/mpeg.bin",
        ]
        .iter()
        .map(|&str| str.to_string())
        .collect()
    }

    let test_data = b"Robert";
    for symbol_table_fname in symbol_tables {
        println!("symbol frequency file {}", symbol_table_fname);
        demonstration1(&symbol_table_fname, test_data)?;
    }

    Ok(())
}

fn demonstration1(fname: &str, test_data: &[u8]) -> Result<(), Error> {
    let mut symbol_file = File::open(fname)?;
    let frequencies = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

    let ans_table = ANSTableUniform::new(frequencies);
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
    let mut x = 1;
    for &symbol in symbols {
        x = ans_table.append_encode64(x, symbol);
    }
    x
}

fn simple_decode(ans_table: &ANSTableUniform, mut val: u64) -> Vec<u8> {
    let mut rval = Vec::new();
    while val > 1 {
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
