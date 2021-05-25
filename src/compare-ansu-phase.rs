use ans_ordering::{catalog_encoding_results, quat_frequencies, quaternary_message_list};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use symbol_table::ANSTableUniform;

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

/// compare the encoding efficiency of various uniform ANS tables generaed using the various possible values for accum_start
fn main() -> Result<(), Box<dyn Error>> {
    let num_quats = 10;
    let sum_frequencies = quat_frequencies().frequencies.iter().sum();

    let workers: Vec<_> = (0..sum_frequencies)
        .map(|phase| {
            thread::spawn(move || {
                let ansu = phased_quat_encoder(phase);

                let report = catalog_encoding_results(
                    &mut quaternary_message_list(num_quats),
                    ansu,
                    &format!("/tmp/q{}.txt", phase),
                )
                .unwrap();
                report
            })
        })
        .collect();

    let results = workers.into_iter().map(|handle| handle.join().unwrap());

    let mut f = File::create("/tmp/q-phases.txt")?;
    for (avg_bits, report) in results {
        print!("{}", report);
        writeln!(f, "{}", avg_bits)?;
    }

    Ok(())
}
