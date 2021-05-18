extern crate symbol_table;
extern crate spmc;

use std::sync::mpsc;

use std::fmt::{Display, Write};
use std::fs::File;
use std::io::Write as W2;
use symbol_table::{ANSTableUniform, SymbolFrequencies};
use std::thread;
use std::error::Error;

pub fn catalog_encoding_results(
    messages: &mut dyn Iterator<Item = Vec<u8>>,
    ansu: &ANSTableUniform,
    output_filename: &str,
) -> Result<(f64,String), Box<dyn Error>> {
    let mut report = String::new();
    let (mut list, sum_bits, sum_prob) = match 2 {
        0=> single_threaded_encode_loop(messages, ansu),
        1=> multi_threaded_encode_loop(messages, ansu),
        _=> multi_threaded_encode_loop_2(&messages.collect::<Vec<Vec<u8>>>(), ansu),
    };

    let average_message_bits = sum_bits / sum_prob;
    writeln!(report,
        "average encoded message length {} = {}/{} for {}",
        average_message_bits, sum_bits, sum_prob, output_filename
    )?;

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

    Ok((average_message_bits,report))
}

fn single_threaded_encode_loop(messages: &mut dyn Iterator<Item=Vec<u8>>, ansu: &ANSTableUniform) -> (Vec<(f64, u64)>, f64, f64) {
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
    (list, sum_bits, sum_prob)
}

/// holy crap, this is worse than the single-threaded version
fn multi_threaded_encode_loop(messages: &mut dyn Iterator<Item=Vec<u8>>, ansu: &ANSTableUniform) -> (Vec<(f64, u64)>, f64, f64) {

    let (mut tx1, rx1) = spmc::channel::<Vec<u8>>();
    let (tx2, rx2) = mpsc::channel();

    let num_threads=8;
    let _workers:Vec<thread::JoinHandle<_>> = (0..num_threads).map(|_| {
        let rx = rx1.clone();
        let tx = tx2.clone();
        let ansu = ansu.clone();
        thread::spawn(move||{
            //let mut count=0;
            while let Ok(message) = rx.recv() {
                let encoded = simple_encode(&ansu, &message);
                let probability = probability_of_message(&ansu, &message);

                tx.send( (probability, encoded) ).unwrap();
                //count+=1;
            }
            //println!("thread processed {} messages", count);
        })
    }).collect();

    let result = thread::spawn(move ||{
        let mut list = Vec::new();
        let mut sum_bits = 0f64;
        let mut sum_prob = 0f64;
        while let Ok((probability,encoded)) = rx2.recv() {
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
        //println!("sum_bits={}; sum_prob={}", sum_bits, sum_prob);
        (list, sum_bits, sum_prob)
    });

    for message in messages {
        tx1.send(message).unwrap();
    }

    drop (tx1);
    drop(tx2);

    result.join().unwrap()

}

fn multi_threaded_encode_loop_2(messages: &[Vec<u8>], ansu: &ANSTableUniform) -> (Vec<(f64, u64)>, f64, f64) {

    let mut num_threads=8;

    let mut work = messages;
    let mut workers = Vec::new();
    while !work.is_empty() {
        let quantum = (work.len() + num_threads - 1) / num_threads;
        let (lhs, rhs) = work.split_at(quantum);
        let span = lhs.to_vec();
        let ansu = ansu.clone();
        let handle = thread::spawn(move || {
            let mut list = Vec::new();
            let mut sum_bits = 0f64;
            let mut sum_prob = 0f64;
            for message in span {
                let encoded = simple_encode(&ansu, &message);
                let probability = probability_of_message(&ansu, &message);
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
            //println!("sum_bits={}; sum_prob={}", sum_bits, sum_prob);
            (list, sum_bits, sum_prob)
        });
        workers.push(handle);
        work = rhs;
        num_threads -=1;
    }

    let mut list = Vec::new();
    let mut sum_bits = 0f64;
    let mut sum_prob = 0f64;
    for worker in workers {
        let (mut piece, partial_sum_bits,partial_sum_prob) = worker.join().unwrap();
        list.append(&mut piece);
        sum_bits += partial_sum_bits;
        sum_prob += partial_sum_prob;
    }

    (list, sum_bits, sum_prob)
}

fn fname_for_unweighted(src: &str) -> String {
    if src.ends_with(".txt") {
        let x = format!("{}_u.txt", &src[..(src.len() - 4)]);
        x
    } else {
        src.to_string() + "_u"
    }
}

pub fn probability_of_message(ansu: &ANSTableUniform, message: &[u8]) -> f64 {
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
    let mut x = 1;
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
