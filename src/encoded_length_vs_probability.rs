use ans_ordering::{
    binary_message_list, debug_dump, polarity_a, probability_of_message, quat_frequencies,
    quaternary_message_list, simple_encode,
};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{Error, Write};
use symbol_table::ANSTableUniform;

pub fn quat_encoder_a() -> ANSTableUniform {
    let freqs = quat_frequencies();
    let rval = ANSTableUniform::new(freqs);
    debug_dump(&rval);
    rval
}

struct Datum {
    encoded: u64,
    probability: f64,
}

impl PartialEq for Datum {
    fn eq(&self, other: &Self) -> bool {
        self.probability == other.probability && self.encoded == other.encoded
    }
}

impl Eq for Datum {}

impl PartialOrd<Datum> for Datum {
    fn partial_cmp(&self, other: &Datum) -> Option<Ordering> {
        let a = self.probability.partial_cmp(&other.probability);
        let b = self.encoded.partial_cmp(&other.encoded);

        a.and_then(|a|
            //b.and_then(|b| Some(a.reverse().then(b)))
            b.map(|b| a.reverse().then(b)))
    }
}

impl Ord for Datum {
    fn cmp(&self, other: &Self) -> Ordering {
        if true {
            return self.partial_cmp(other).unwrap();
        }

        let a: Ordering = if self.probability < other.probability {
            // this ignores NaNs and stuff
            Ordering::Less
        } else if self.probability > other.probability {
            Ordering::Greater
        } else {
            Ordering::Equal
        };
        let b = self.encoded.cmp(&other.encoded);

        a.then(b)
    }
}

fn main() -> Result<(), Error> {
    mission1()?;
    mission2()?;

    Ok(())
}

fn mission1() -> Result<(), Error> {
    let ansu = quat_encoder_a();

    let mut catalog = Vec::new();

    for msg in quaternary_message_list(10) {
        let encoded = simple_encode(&ansu, &msg);
        let probability = probability_of_message(&ansu, &msg);
        catalog.push(Datum {
            encoded,
            probability,
        });
    }

    catalog.sort();

    let ofname = "/tmp/by-prob-4.txt";
    let mut f = File::create(ofname)?;

    for datum in catalog {
        writeln!(f, "{}\t{}", datum.encoded, datum.probability)?;
    }
    println!("wrote {}", ofname);

    Ok(())
}

fn mission2() -> Result<(), Error> {
    let ansu = polarity_a();

    let mut catalog = Vec::new();

    for msg in binary_message_list(20) {
        let encoded = simple_encode(&ansu, &msg);
        let probability = probability_of_message(&ansu, &msg);
        catalog.push(Datum {
            encoded,
            probability,
        });
    }

    catalog.sort();

    let ofname = "/tmp/by-prob-2.txt";
    let mut f = File::create(ofname)?;

    for datum in catalog {
        writeln!(f, "{}\t{}", datum.encoded, datum.probability)?;
    }
    println!("wrote {}", ofname);

    Ok(())
}
