extern crate symbol_table;

use std::fs::File;
use std::io::{Error, Read, Write};
use std::{io, panic};
use symbol_table::{SymbolFrequencies, StreamingANSUniform};

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

fn demo_suite1(
    symbol_fname: &str,
    message: &[u8],
    backfill_missing_symbols: bool,
) -> Result<(), Error> {
    let message1 = message;
    println!("  message.len() = {}", message.len());

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
    let compressed = uans.encode(message.iter().rev());

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
