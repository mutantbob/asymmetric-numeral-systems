use ans_ordering::{
    binary_message_list, catalog_encoding_results, polarity_a, polarity_b, polarity_c,
};
use std::error::Error;

/// generate a catalog of the encoded result for all the 20bit messages using three different ANS encoding tables
fn main() -> Result<(), Box<dyn Error>> {
    let num_bits = 20;

    catalog_encoding_results(
        &mut binary_message_list(num_bits),
        polarity_a(),
        "/tmp/a.txt",
    )?;

    catalog_encoding_results(
        &mut binary_message_list(num_bits),
        polarity_b(),
        "/tmp/b.txt",
    )?;

    catalog_encoding_results(
        &mut binary_message_list(num_bits),
        polarity_c(),
        "/tmp/c.txt",
    )?;

    Ok(())
}
