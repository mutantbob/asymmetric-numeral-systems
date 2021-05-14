use std::fs::File;
use std::io::{Error, Read};

use symbol_table::SymbolFrequencies;

pub fn slurp(fname: &str) -> Result<Vec<u8>, Error> {
    let mut f = File::open(fname)?;
    let mut payload = Vec::new();
    let _count = f.read_to_end(&mut payload)?;
    Ok(payload)
}

pub fn scale_frequencies(
    num_bits: u8,
    raw: &SymbolFrequencies,
    verbose: bool,
) -> SymbolFrequencies {
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
