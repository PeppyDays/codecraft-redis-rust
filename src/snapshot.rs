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
        RdbFileReader {
            reader: Mutex::new(BufReader::new(file)),
        }
    }

    fn initialize(&self) {
        let mut reader = self.reader.lock().unwrap();
        reader.seek(SeekFrom::Start(0)).unwrap();
    }

    fn parse_header(&self) -> String {
        self.initialize();

        let mut buffer = [0u8; 9];
        self.reader.lock().unwrap().read_exact(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }

    // fn read_byte(&mut self) -> Result<u8> {
    //     let mut buf = [0u8; 1];
    //     self.reader.read_exact(&mut buf)?;
    //     Ok(buf[0])
    // }

    // fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
    //     let mut buf = vec![0u8; count];
    //     self.reader.read_exact(&mut buf)?;
    //     Ok(buf)
    // }
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
