use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::Mutex;

pub struct RdbFileReader {
    reader: Mutex<BufReader<File>>,
}

impl RdbFileReader {
    pub fn new(file: File) -> Self {
        let reader = RdbFileReader {
            reader: Mutex::new(BufReader::new(file)),
        };
        reader.initialize();
        reader
    }

    fn initialize(&self) {
        let mut reader = self.reader.lock().unwrap();
        reader.seek(SeekFrom::Start(0)).unwrap();
    }

    fn parse_header(&self) -> String {
        let buffer = self.read_bytes(9).unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }

    fn parse_metadata(&self) -> HashMap<String, String> {
        let indicator = self.read_byte().unwrap();
        if indicator != 0xFA {
            panic!("OMG");
        }
        let mut metadata = HashMap::new();
        let name = String::from_utf8_lossy(&self.read_bytes(self.read_size().unwrap()).unwrap())
            .to_string();
        let value = String::from_utf8_lossy(&self.read_bytes(self.read_size().unwrap()).unwrap())
            .to_string();
        metadata.insert(name, value);
        metadata
    }

    fn read_byte(&self) -> Result<u8, anyhow::Error> {
        let mut buffer = [0u8; 1];
        self.reader.lock().unwrap().read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    fn read_bytes(&self, count: usize) -> Result<Vec<u8>, anyhow::Error> {
        let mut buffer = vec![0u8; count];
        self.reader.lock().unwrap().read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn read_size(&self) -> Result<usize, anyhow::Error> {
        let first_byte = self.read_byte()?;
        let first_two_bits = (first_byte >> 6) & 0b11;
        let remaining_bites = first_byte & 0b00111111;
        match first_two_bits {
            0b00 => Ok(remaining_bites as usize),
            0b01 => {
                let second_bytes = self.read_byte()?;
                Ok(((remaining_bites as usize) << 6) + second_bytes as usize)
            }
            0b10 => {
                let next_four_bytes = self.read_bytes(5)?;
                Ok(next_four_bytes
                    .iter()
                    .fold(0usize, |acc, &b| (acc << 8) | b as usize))
            }
            0b11 => match remaining_bites {
                0xC0 => Ok(1_usize),
                0xC1 => Ok(2_usize),
                0xC2 => Ok(4_usize),
                _ => unimplemented!(),
            },
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod specs_for_load {
    use std::io::Write;

    use tempfile::tempfile;

    use super::*;

    #[test]
    fn sut_parses_header_of_rdb_correctly() {
        // Arrange
        let mut file = tempfile().unwrap();
        file.write_all(header()).unwrap();

        let sut = RdbFileReader::new(file);

        // Act
        let actual = sut.parse_header();

        // Assert
        let expected = "REDIS0011";
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_parses_metadata_of_rdb_correctly() {
        // Arrange
        let mut file = tempfile().unwrap();
        file.write_all(header()).unwrap();
        file.write_all(metadata()).unwrap();

        let sut = RdbFileReader::new(file);
        let _header = sut.parse_header();

        // Act
        let actual = sut.parse_metadata();

        // Assert
        let expected = HashMap::from_iter([("redis-ver".to_string(), "6.0.16".to_string())]);
        assert_eq!(actual, expected);
    }

    fn header() -> &'static [u8] {
        // REDIS0011
        &[0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31]
    }

    fn metadata() -> &'static [u8] {
        &[
            0xFA, 0x09, 0x72, 0x65, 0x64, 0x69, 0x73, 0x2D, 0x76, 0x65, 0x72, 0x06, 0x36, 0x2E,
            0x30, 0x2E, 0x31, 0x36,
        ]
    }
}
