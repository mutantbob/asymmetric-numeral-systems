use ans_ordering::{debug_dump, quat_frequencies};
use symbol_table::ANSTableUniform;

/// Dump the encoding tables we generate when using a different accum_start param
/// to visually inspect them
fn main() {
    let freqs = quat_frequencies();

    let sum_frequencies = freqs.frequencies.iter().sum();
    let (alt_encode, _decode) =
        ANSTableUniform::build_tables(&freqs.frequencies, sum_frequencies, 0);

    println!("default phase:");
    let mut ansu = ANSTableUniform::new(freqs);
    debug_dump(&ansu);
    println!();

    //

    println!("phase=0 :");
    ansu.encode = alt_encode;

    debug_dump(&ansu);
}
