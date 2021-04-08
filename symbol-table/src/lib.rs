use std::io::{Error, Read};

pub struct SymbolFrequencies {
    pub frequencies: [u32; 256],
}

impl SymbolFrequencies {
    pub fn new() -> SymbolFrequencies {
        SymbolFrequencies {
            frequencies: [0; 256],
        }
    }

    pub fn scan_file(&mut self, f: &mut dyn Read) -> Result<(), Error> {
        let mut buffer = [0; 4 << 10];

        loop {
            let count = f.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            for &symbol in &buffer[..count] {
                self.frequencies[symbol as usize] += 1;
            }
        }
        Ok(())
    }
}

impl Default for SymbolFrequencies {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
