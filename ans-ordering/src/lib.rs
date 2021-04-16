extern crate symbol_table;

use std::fmt::{Display, Write};
use std::fs::File;
use std::io::Error;
use std::io::Write as W2;
use symbol_table::{ANSTableUniform, SymbolFrequencies};

pub fn catalog_encoding_results(
    messages: &mut dyn Iterator<Item = Vec<u8>>,
    ansu: &ANSTableUniform,
    output_filename: &str,
) -> Result<f64, Error> {
    let mut list = Vec::new();
    let mut sum_bits = 0f64;
    let mut sum_prob = 0f64;
    for message in messages {
        let encoded = simple_encode(ansu, &message);
        let probability = probability_of_message(ansu, &message);
        let num_encoded_bits = (1.max(encoded) as f64).log2();
        sum_bits += probability * num_encoded_bits;
        sum_prob += probability;
        if sum_bits.is_nan() {
            panic!("sum_bits += {} * {}", probability, num_encoded_bits);
        }
        if sum_prob.is_nan() {
            panic!("sum_prob += {}", probability);
        }
        list.push((probability, encoded));
    }

    let average_message_bits = sum_bits / sum_prob;
    println!(
        "average encoded message length {} = {}/{} for {}",
        average_message_bits, sum_bits, sum_prob, output_filename
    );

    list.sort_unstable_by(|(_, encoded_a), (_, encoded_b)| (*encoded_a).cmp(encoded_b));

    {
        let mut f = File::create(output_filename)?;
        let mut x = 0f64;
        for (prob, encoded) in list.iter() {
            x += prob;
            writeln!(f, "{}\t{}", x, encoded)?;
        }
    }

    let mut f = File::create(fname_for_unweighted(output_filename))?;
    for (_, encoded) in list {
        writeln!(f, "{}", encoded)?;
    }

    Ok(average_message_bits)
}

fn fname_for_unweighted(src: &str) -> String {
    if src.ends_with(".txt") {
        let x = format!("{}_u.txt", &src[..(src.len() - 4)]);
        x
    } else {
        src.to_string() + "_u"
    }
}

fn probability_of_message(ansu: &ANSTableUniform, message: &[u8]) -> f64 {
    let mut rval = 1f64;
    let sum_frequencies = ansu.sum_frequencies as f64;
    for &symbol in message {
        let freq = ansu.frequencies[symbol as usize] as f64;
        //println!("{} *= {}/{}", rval, freq, sum_frequencies);
        rval *= freq / sum_frequencies;
        if rval.is_nan() {
            panic!("rval *= {} / {}", freq, sum_frequencies);
        }
    }
    rval
}

pub fn simple_encode(ansu: &ANSTableUniform, message: &[u8]) -> u64 {
    let mut x = 0;
    for &symbol in message {
        x = ansu.append_encode64(x, symbol);
    }
    x
}

fn join<I, T, D>(iter: I, separator: D) -> String
where
    T: Display,
    D: Display,
    I: Iterator<Item = T>,
{
    match iter.fold(None, |a: Option<String>, b| {
        Some(match a {
            None => format!("{}", &b),
            Some(mut a) => {
                write!(a, "{}{}", separator, b).unwrap();
                a
            }
        })
    }) {
        None => String::new(),
        Some(rval) => rval,
    }
}

pub fn debug_dump(p0: &ANSTableUniform) {
    for jump_table in p0.encode.iter().filter(|v| !v.is_empty()) {
        println!("{}", join(jump_table.iter(), ","));
    }
}

//

pub fn polarity_a() -> ANSTableUniform {
    let mut symbol_frequencies = SymbolFrequencies::new();
    symbol_frequencies.frequencies[0] = 3;
    symbol_frequencies.frequencies[1] = 1;
    ANSTableUniform::new(symbol_frequencies)
}

/// specially constructed to test how the ordering of the uniform encoding tables affects compression efficiency
pub fn polarity_b() -> ANSTableUniform {
    let mut frequencies = [0; 256];
    frequencies[0] = 3;
    frequencies[1] = 1;
    ANSTableUniform {
        frequencies,
        sum_frequencies: 4,
        encode: vec![vec![1, 2, 3], vec![0]], // this is a special encoding that is different from our original calculations
        decode: vec![],
        verbose: false,
    }
}

/// specially constructed to test how the ordering of the uniform encoding tables affects compression efficiency
pub fn polarity_c() -> ANSTableUniform {
    let mut frequencies = [0; 256];
    frequencies[0] = 3;
    frequencies[1] = 1;
    ANSTableUniform {
        frequencies,
        sum_frequencies: 4,
        encode: vec![vec![0, 1, 2], vec![3]], // this is a special encoding that is different from our original calculations
        decode: vec![],
        verbose: false,
    }
}

pub fn binary_expand(packed_bits: i32, num_bits: u8) -> Vec<u8> {
    (0..num_bits)
        .map(|pos| 0 != packed_bits & (1 << pos))
        .map(|bit| if bit { 1 } else { 0 })
        .collect()
}

pub fn binary_message_list(num_bits: u8) -> Box<dyn Iterator<Item = Vec<u8>>> {
    Box::new((0..(1 << num_bits)).map(move |message| binary_expand(message, num_bits)))
}

//

fn quaternary_expand(packed_bits: i32, num_quats: u8) -> Vec<u8> {
    (0..num_quats)
        .map(|pos| ((packed_bits >> (2 * pos)) & 3) as u8)
        .collect()
}

pub fn quaternary_message_list(num_quats: u8) -> Box<dyn Iterator<Item = Vec<u8>>> {
    Box::new((0..(1 << (2 * num_quats))).map(move |message| quaternary_expand(message, num_quats)))
}

pub fn quat_frequencies() -> SymbolFrequencies {
    let mut freqs = SymbolFrequencies::new();
    freqs.frequencies[0] = 1;
    freqs.frequencies[1] = 2;
    freqs.frequencies[2] = 4;
    freqs.frequencies[3] = 8;
    freqs
}
