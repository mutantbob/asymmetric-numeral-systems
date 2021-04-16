use ans_ordering::{binary_expand, debug_dump, polarity_a, polarity_b, simple_encode};
use std::cmp::Ordering;
use std::fs::File;
use std::io::Error;
use std::io::Write;

/// Iterate through all the possible 16-bit messages and encode them using two different ANS tables.
/// Output the resulting encdings to /tmp/a.txt and /tmp/b.txt and stdout.
/// stdout can be used to build a scatter plot.
fn main() -> Result<(), Error> {
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
