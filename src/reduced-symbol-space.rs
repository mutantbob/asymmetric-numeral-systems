extern crate symbol_table;

mod cliches;

use crate::cliches::slurp;
use cliches::scale_frequencies;
use std::fs::File;
use std::io::Error;
use symbol_table::{StreamingANSUniform, SymbolFrequencies};

fn main() -> Result<(), Error> {
    {
        println!("#\tat the mountains of madness");
        let symbol_fname = "../test-data/out/atmm.bin";
        let message_fname = "../test-data/at-the-mountains-of-madness.html";
        let message2 = slurp(message_fname)?;
        analyze(symbol_fname, &message2)?;
    }
    {
        println!("#\tdream quest of unknown kadath");
        let symbol_fname = "../test-data/out/dq.bin";
        let message_fname = "../test-data/dream-quest.html";
        let message2 = slurp(message_fname)?;
        analyze(symbol_fname, &message2)?;
    }

    Ok(())
}

fn analyze(symbol_fname: &str, message2: &[u8]) -> Result<(), Error> {
    println!("orig\t{}", message2.len());

    {
        let frequencies = build_flat_frequencies(&message2);

        let symbol_count = frequencies.iter().filter(|&&freq| freq != 0).count();
        println!("#\t\t{} distinct symbols in message", symbol_count);

        {
            let predicted = (message2.len() as f32) * (symbol_count as f32).ln() / 256f32.ln();
            println!(
                "#\t\tswitching to a flat symbol table of only used symbols will bring it near {}",
                predicted
            );
        }

        let freqs = SymbolFrequencies { frequencies };
        let ansu = StreamingANSUniform::new(freqs, 16, 2);
        let encoded = ansu.encode(message2.iter().rev(), 1);

        println!("flat\t{}", encoded.len());
    }

    {
        let mut symbol_file = File::open(symbol_fname)?;
        let freqs = SymbolFrequencies::parse_binary_symbol_table(&mut symbol_file)?;

        let freqs = scale_frequencies(16, &freqs, false);

        let ansu = StreamingANSUniform::new(freqs, 16, 2);
        let encoded_well = ansu.encode(message2.iter().rev(), 1);

        println!(
            "#\t\tencoding with a symbol table with asymmetric frequencies\nmatching\t{}",
            encoded_well.len()
        );
    }

    Ok(())
}

fn build_flat_frequencies(message2: &[u8]) -> [u32; 256] {
    let mut frequencies = [0; 256];
    for &symbol in message2 {
        frequencies[symbol as usize] = 1;
    }

    frequencies
}
