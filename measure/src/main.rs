/*
count the frequency of symbols (bytes) in a file.
This frequency table will later be used by Asymmetrical Numerical System compression tools

Usage:
  $0 file1 [file2...] [ -o freqs.bin | -O freqs.txt ]

  If no output file is specified, a verbose text readout will be sent to stdout
 */

extern crate byteorder;
extern crate symbol_table;

use byteorder::BigEndian;
use byteorder::WriteBytesExt;
use std::env;
use std::fs::File;
use std::io::{Error, Write};
use symbol_table::SymbolFrequencies;

trait SymbolTableSink {
    fn output(&mut self, table: &[u32]) -> Result<(), Error>;
}

//

struct StdoutSymbolTableSink {}

impl SymbolTableSink for StdoutSymbolTableSink {
    fn output(&mut self, table: &[u32]) -> Result<(), Error> {
        for (symbol, &freq) in table.iter().enumerate() {
            if freq > 0 {
                println!("[{}]\tx {}", symbol, freq);
            }
        }

        Ok(())
    }
}

//

struct BinarySymbolTableSink<T: Write> {
    sink: T,
}

impl<T: Write> BinarySymbolTableSink<T> {
    fn new(sink: T) -> BinarySymbolTableSink<T> {
        BinarySymbolTableSink { sink }
    }
}

impl<T: Write> SymbolTableSink for BinarySymbolTableSink<T> {
    fn output(&mut self, table: &[u32]) -> Result<(), Error> {
        for &freq in table {
            self.sink.write_u32::<BigEndian>(freq)?;
        }
        println!("wrote binary results");
        Ok(())
    }
}

//

struct TextSymbolTableSink<T: Write> {
    sink: T,
}

impl<T: Write> TextSymbolTableSink<T> {
    fn new(sink: T) -> TextSymbolTableSink<T> {
        TextSymbolTableSink { sink }
    }
}

impl<T: Write> SymbolTableSink for TextSymbolTableSink<T> {
    fn output(&mut self, table: &[u32]) -> Result<(), Error> {
        for (sym, freq) in table.iter().enumerate() {
            writeln!(self.sink, "{} {}", sym, &freq)?;
        }

        Ok(())
    }
}

//

struct Mission {
    fnames: Vec<String>,
    output: Box<dyn SymbolTableSink>,
}

//

//

fn main() -> Result<(), Error> {
    let mut table = SymbolFrequencies::new();

    let args = env::args();
    let mut args = args.skip(1);

    let mut mission = args_to_mission(&mut args)?;

    for fname in &mission.fnames {
        let f = File::open(&fname);
        println!("scanning symbols from {}", &fname);

        if let Err(e) = f.and_then(|mut f| table.scan_file(&mut f)) {
            println!("malfunction reading {} because {:?}", &fname, e);
        };
    }

    mission.output.output(&table.frequencies)?;

    Ok(())
}

fn args_to_mission(args: &mut dyn Iterator<Item = String>) -> Result<Mission, Error> {
    let mut output: Box<dyn SymbolTableSink> = Box::new(StdoutSymbolTableSink {});
    let mut fnames: Vec<String> = Vec::new();
    //let mut output_file = None;

    loop {
        let arg = match args.next() {
            None => {
                break;
            }
            Some(arg) => arg,
        };

        if "-o" == arg {
            let ofname = args.next().unwrap();
            //output_file = Some(File::create(ofname)?);
            output = Box::new(BinarySymbolTableSink::new(File::create(ofname)?));
        } else if "-O" == arg {
            let ofname = args.next().unwrap();
            //output_file = Some(File::create(ofname)?);
            output = Box::new(TextSymbolTableSink::new(File::create(ofname)?));
        } else {
            fnames.push(arg);
        }
    }

    Ok(Mission { fnames, output })
}
