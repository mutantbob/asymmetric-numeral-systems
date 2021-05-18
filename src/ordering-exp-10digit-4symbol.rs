use ans_ordering::{
    catalog_encoding_results, debug_dump, quat_frequencies, quaternary_message_list,
};
use std::io::Error;
use std::thread;
use std::thread::JoinHandle;
use symbol_table::{ANSTableUniform, SymbolFrequencies};

/// Create sorted encoding catalogs of 10-digit encodings from a 4-symbol alphabet
/// using various encoding tables.  These catalogs will be analyzed to evaluate
/// their efficiency.
fn main() -> Result<(), Error> {
    let num_quats = 10;
    let params: Vec<(Box<dyn Fn() -> ANSTableUniform + Send>, &'static str)> = vec![
        (Box::new(quat_encoder_a), "/tmp/qa.txt"),
        (Box::new(quat_encoder_b), "/tmp/qb.txt"),
        (Box::new(quat_encoder_c), "/tmp/qc.txt"),
        (Box::new(quat_encoder_d), "/tmp/qd.txt"),
        (Box::new(quat_encoder_e), "/tmp/qe.txt"),
        (Box::new(quat_encoder_f), "/tmp/qf.txt"),
    ];
    let workers = params
        .into_iter()
        .map(|(encoder, fname)| {
            thread::spawn(move || {
                catalog_encoding_results(&mut quaternary_message_list(num_quats), &encoder(), fname)
                    .unwrap()
            })
        })
        .collect::<Vec<JoinHandle<_>>>();

    for worker in workers {
        let (_avg_bits, report) = worker.join().unwrap();
        print!("{}", report);
    }

    Ok(())
}

pub fn quat_encoder_a() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let rval = ANSTableUniform::new(freqs);
    debug_dump(&rval);
    rval
}

/// range table instead of uniform
pub fn quat_encoder_b() -> ANSTableUniform {
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
pub fn quat_encoder_c() -> ANSTableUniform {
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
