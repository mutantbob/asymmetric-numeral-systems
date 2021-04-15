extern crate symbol_table;

use std::cmp::Ordering;
use std::fmt::{Display, Write};
use std::fs::File;
use std::io::Error;
use std::io::Write as W2;
use symbol_table::{ANSTableUniform, SymbolFrequencies};

fn catalog_encoding_results(
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

fn simple_encode(ansu: &ANSTableUniform, message: &[u8]) -> u64 {
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

fn debug_dump(p0: &ANSTableUniform) {
    for jump_table in p0.encode.iter().filter(|v| !v.is_empty()) {
        println!("{}", join(jump_table.iter(), ","));
    }
}

//

fn main() -> Result<(), Error> {
    if false {
        mission1()?;
    }

    if false {
        mission2()?;
    }

    if true {
        mission3()?;
    }

    if false {
        mission4();
    }

    if false {
        mission5()?;
    }

    Ok(())
}

fn mission1() -> Result<(), Error> {
    let ansu_a = polarity_a();
    let ansu_b = polarity_b();

    if false {
        debug_dump(&ansu_a);
        debug_dump(&ansu_b);
        return Ok(());
    }

    let mut a_list = Vec::new();
    let mut b_list = Vec::new();

    let num_bits = 16;
    for message in 0..(1 << num_bits) {
        let message_fat = binary_expand(message, num_bits);
        let a = simple_encode(&ansu_a, &message_fat);
        let b = simple_encode(&ansu_b, &message_fat);
        let dir = match a.cmp(&b) {
            Ordering::Less => "<",
            Ordering::Equal => "=",
            Ordering::Greater => ">",
        };
        println!("{}\t{}\t{}{}", a, b, dir, message_fat.first().unwrap());

        a_list.push(a);
        b_list.push(b);
    }

    a_list.sort();
    b_list.sort();

    let mut f_a = File::create("/tmp/a.txt")?;
    for a in a_list {
        writeln!(f_a, "{}", a)?;
    }

    let mut f_b = File::create("/tmp/b.txt")?;
    for b in b_list {
        writeln!(f_b, "{}", b)?;
    }

    Ok(())
}

//

fn polarity_a() -> ANSTableUniform {
    let mut symbol_frequencies = SymbolFrequencies::new();
    symbol_frequencies.frequencies[0] = 3;
    symbol_frequencies.frequencies[1] = 1;
    ANSTableUniform::new(symbol_frequencies)
}

/// specially constructed to test how the ordering of the uniform encoding tables affects compression efficiency
fn polarity_b() -> ANSTableUniform {
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

fn binary_expand(packed_bits: i32, num_bits: u8) -> Vec<u8> {
    (0..num_bits)
        .map(|pos| 0 != packed_bits & (1 << pos))
        .map(|bit| if bit { 1 } else { 0 })
        .collect()
}

fn binary_message_list(num_bits: u8) -> Box<dyn Iterator<Item = Vec<u8>>> {
    Box::new((0..(1 << num_bits)).map(move |message| binary_expand(message, num_bits)))
}

fn mission2() -> Result<(), Error> {
    let num_bits = 20;

    catalog_encoding_results(
        &mut binary_message_list(num_bits),
        &polarity_a(),
        "/tmp/a.txt",
    )?;

    catalog_encoding_results(
        &mut binary_message_list(num_bits),
        &polarity_b(),
        "/tmp/b.txt",
    )?;

    Ok(())
}

//

fn quaternary_expand(packed_bits: i32, num_quats: u8) -> Vec<u8> {
    (0..num_quats)
        .map(|pos| ((packed_bits >> (2 * pos)) & 3) as u8)
        .collect()
}

fn quaternary_message_list(num_quats: u8) -> Box<dyn Iterator<Item = Vec<u8>>> {
    Box::new((0..(1 << (2 * num_quats))).map(move |message| quaternary_expand(message, num_quats)))
}

fn quat_frequencies() -> SymbolFrequencies {
    let mut freqs = SymbolFrequencies::new();
    freqs.frequencies[0] = 1;
    freqs.frequencies[1] = 2;
    freqs.frequencies[2] = 4;
    freqs.frequencies[3] = 8;
    freqs
}

fn quat_encoder_a() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let rval = ANSTableUniform::new(freqs);
    debug_dump(&rval);
    rval
}

/// range table instead of uniform
fn quat_encoder_b() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let sum_frequencies = freqs.frequencies.iter().sum();
    let encode = generate_range_encoder_increasing(&freqs);
    let rval = ANSTableUniform {
        frequencies: freqs.frequencies,
        sum_frequencies,
        encode,
        decode: vec![], // unused
        verbose: false,
    };
    debug_dump(&rval);
    rval
}

fn generate_range_encoder_increasing(freqs: &SymbolFrequencies) -> Vec<Vec<u32>> {
    let mut encode = Vec::new();
    let mut cursor = 0;
    for (_symbol, &freq) in freqs.frequencies.iter().enumerate() {
        if freq < 1 {
            break;
        }
        let mut nexts = Vec::new();
        for _ in 0..freq {
            nexts.push(cursor);
            cursor += 1;
        }
        encode.push(nexts);
    }
    encode
}

/// range table instead of uniform, but reversed from b
fn quat_encoder_c() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let sum_frequencies = freqs.frequencies.iter().sum();
    let rval = ANSTableUniform {
        frequencies: freqs.frequencies,
        sum_frequencies,
        encode: generate_range_encoder_semidecreasing(&freqs),
        decode: vec![], // unused
        verbose: false,
    };
    debug_dump(&rval);
    rval
}

fn generate_range_encoder_semidecreasing(freqs: &SymbolFrequencies) -> Vec<Vec<u32>> {
    let mut encode = Vec::new();
    let mut cursor: u32 = freqs.frequencies.iter().sum();
    for (_symbol, &freq) in freqs.frequencies.iter().enumerate() {
        if freq < 1 {
            break;
        }
        let mut nexts = Vec::new();
        for i in 0..freq {
            let x = cursor - freq + i;
            nexts.push(x);
        }
        cursor -= freq;
        encode.push(nexts);
    }
    encode
}

fn quat_encoder_d() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let mut rval = ANSTableUniform::new(freqs);
    for per_symbol in rval.encode.iter_mut() {
        for next in per_symbol.iter_mut() {
            *next = rval.sum_frequencies - *next - 1; // flip all the encodings
        }
        per_symbol.sort();
    }
    debug_dump(&rval);
    rval
}

fn quat_encoder_e() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let sum_frequencies = freqs.frequencies.iter().sum();
    let (encode, decode) = ANSTableUniform::build_tables(&freqs.frequencies, sum_frequencies, 0);
    let rval = ANSTableUniform {
        frequencies: freqs.frequencies,
        sum_frequencies,
        encode,
        decode,
        verbose: false,
    };
    debug_dump(&rval);
    rval
}

fn quat_encoder_f() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let sum_frequencies = freqs.frequencies.iter().sum();
    let encode = vec![
        vec![0],
        vec![5, 9],
        vec![2, 7, 11, 13],
        vec![1, 3, 4, 6, 8, 10, 12, 14],
    ];
    let rval = ANSTableUniform {
        frequencies: freqs.frequencies,
        sum_frequencies,
        encode,
        decode: vec![], // unused
        verbose: false,
    };
    debug_dump(&rval);
    rval
}

fn mission3() -> Result<(), Error> {
    let num_quats = 10;

    catalog_encoding_results(
        &mut quaternary_message_list(num_quats),
        &quat_encoder_a(),
        "/tmp/qa.txt",
    )?;

    catalog_encoding_results(
        &mut quaternary_message_list(num_quats),
        &quat_encoder_b(),
        "/tmp/qb.txt",
    )?;

    catalog_encoding_results(
        &mut quaternary_message_list(num_quats),
        &quat_encoder_c(),
        "/tmp/qc.txt",
    )?;

    catalog_encoding_results(
        &mut quaternary_message_list(num_quats),
        &quat_encoder_d(),
        "/tmp/qd.txt",
    )?;

    catalog_encoding_results(
        &mut quaternary_message_list(num_quats),
        &quat_encoder_e(),
        "/tmp/qe.txt",
    )?;

    catalog_encoding_results(
        &mut quaternary_message_list(num_quats),
        &quat_encoder_f(),
        "/tmp/qf.txt",
    )?;

    Ok(())
}

fn mission4() {
    let freqs = quat_frequencies();

    let sum_frequencies = freqs.frequencies.iter().sum();
    let (alt_encode, _decode) =
        ANSTableUniform::build_tables(&freqs.frequencies, sum_frequencies, sum_frequencies / 2);

    let mut ansu = ANSTableUniform::new(freqs);
    debug_dump(&ansu);

    //

    ansu.encode = alt_encode;

    debug_dump(&ansu);
}

//
//
//

fn mission5() -> Result<(), Error> {
    let num_quats = 10;
    let sum_frequencies = quat_frequencies().frequencies.iter().sum();
    let mut results = Vec::new();
    for phase in 0..sum_frequencies {
        let ansu = phased_quat_encoder(phase);

        let avg_bits = catalog_encoding_results(
            &mut quaternary_message_list(num_quats),
            &ansu,
            &format!("/tmp/q{}.txt", phase),
        )?;
        results.push(avg_bits);
    }

    let mut f = File::create("/tmp/q-phases.txt")?;
    for avg_bits in results {
        writeln!(f, "{}", avg_bits)?;
    }

    Ok(())
}

fn phased_quat_encoder(phase: u32) -> ANSTableUniform {
    let freqs = quat_frequencies();

    let mut ansu = ANSTableUniform::new(freqs);

    let sum_frequencies = ansu.sum_frequencies;
    let (alt_encode, alt_decode) =
        ANSTableUniform::build_tables(&ansu.frequencies, sum_frequencies, phase);

    ansu.encode = alt_encode;
    ansu.decode = alt_decode;
    ansu
}
