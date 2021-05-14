extern crate symbol_table;

mod cliches;

use std::error::Error;
use std::fs::File;
use std::io::Write as Write1;
use std::{io, panic};

use symbol_table::{StreamingANSUniform, SymbolFrequencies};

use crate::cliches::{scale_frequencies, slurp};
use std::fmt::Write;
use std::thread::spawn;

//

fn main() -> Result<(), std::io::Error> {
    {
        let backfill_missing_symbols = false;

        let message2 = slurp("../test-data/at-the-mountains-of-madness.html")?;
        let message3 = slurp("../test-data/dream-quest.html")?;
        let message4 = slurp("../test-data/iso13818-2.pdf")?;

        let mut per_table = Vec::new();
        for symbol_fname in &[
            "../test-data/out/atmm.bin",
            "../test-data/out/dq.bin",
            "../test-data/out/mpeg.bin",
        ] {
            println!("symbol frequency file {}", symbol_fname);

            let mut futures = Vec::new();
            let messages: Vec<Vec<u8>> = vec![
                b"what is a man, but a miserable pile of secrets?".to_vec(),
                message2.clone(),
                message3.clone(),
                message4.clone(),
            ];
            for message in messages {
                let symbol_fname = symbol_fname.to_string();
                let future = spawn(move || {
                    match demo_suite1(&symbol_fname, &message, backfill_missing_symbols) {
                        Err(e) => format!("malfunction {:?}\n", e),
                        Ok(msg) => msg,
                    }
                });

                futures.push(future);
            }

            per_table.push((symbol_fname.to_string(), futures));
        }

        for (symbol_fname, futures) in per_table {
            println!("symbol frequency file {}", symbol_fname);

            for future in futures {
                let result = future.join();

                match result {
                    Ok(msg) => {
                        println!("{}", msg);
                    }
                    Err(e) => {
                        let msg = if let Some(msg) = e.downcast_ref::<&'static str>() {
                            msg.to_string()
                        } else if let Some(msg) = e.downcast_ref::<String>() {
                            msg.to_string()
                        } else {
                            format!("?? {:?}", e)
                        };
                        println!("panic {:?}", msg);
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
) -> Result<String, Box<dyn Error>> {
    let mut rval = String::new();
    writeln!(rval, "  message.len() = {}", message.len())?;

    //
    {
        let payload = message;
        //let payload:&[u8] = &payload;
        demonstration1(
            symbol_fname,
            &payload,
            16,
            2,
            backfill_missing_symbols,
            &mut rval,
        )?;
        demonstration1(
            symbol_fname,
            &payload,
            24,
            2,
            backfill_missing_symbols,
            &mut rval,
        )?;
        demonstration1(
            symbol_fname,
            &payload,
            32,
            2,
            backfill_missing_symbols,
            &mut rval,
        )?;
    }

    Ok(rval)
}

fn demonstration1(
    symbol_fname: &str,
    message: &[u8],
    underflow_bits: u8,
    bytes_to_stream: u8,
    backfill_missing_symbols: bool,
    sink: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    writeln!(sink, "demonstration(,,{})", underflow_bits)?;
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

    demonstration1a(&mut uans, message, underflow_bits, sink)?;
    Ok(())
}

fn demonstration1a(
    uans: &mut StreamingANSUniform,
    message: &[u8],
    underflow_bits: u8,
    sink: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let iv = 1;
    let compressed = uans.encode(message.iter().rev(), iv);

    writeln!(
        sink,
        "compressed to {} bytes (UB={})",
        compressed.len(),
        underflow_bits
    )?;

    let uncompressed = uans.decode(&compressed, iv).unwrap();

    if false {
        io::stdout().write_all(&uncompressed)?;
        println!();
    }

    assert!(uncompressed == message, "encode/decode mismatch");
    Ok(())
}
