use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter::from_fn;
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
        let _ = reader.header();
        reader
    }

    fn initialize(&self) {
        let mut reader = self.reader.lock().unwrap();
        reader.seek(SeekFrom::Start(0)).unwrap();
    }

    fn header(&self) -> String {
        let buffer = self.read_bytes(9).unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }

    pub fn entries(&self) -> impl Iterator<Item = (usize, String, String, Option<u128>)> {
        let mut db = 0;

        from_fn(move || {
            loop {
                let entry_type = self.read_byte().ok()?;

                match entry_type {
                    0xFA => {
                        // metadata
                        let _key = self.read_string();
                        let _value = self.read_string();
                        continue;
                    }
                    0xFE => {
                        // DB selector
                        db = self.read_byte().ok()? as usize;
                        continue;
                    }
                    0xFB => {
                        // hash table size
                        let _hash_table_size = self.read_size().ok()?;
                        let _expires_hash_table_size = self.read_size().ok()?;
                        continue;
                    }
                    0x00 => {
                        // entry without expiration
                        let key = self.read_string().ok()?;
                        let value = self.read_string().ok()?;
                        return Some((db, key, value, None));
                    }
                    0xFC => {
                        // Entry with milliseconds expiry
                        let expiry = self.read_expiry_in_millis().ok()?;
                        let _encoding = self.read_byte().ok()?;
                        let key = self.read_string().ok()?;
                        let value = self.read_string().ok()?;
                        return Some((db, key, value, Some(expiry)));
                    }
                    0xFD => {
                        // Entry with seconds expiry
                        let expiry = self.read_expiry_in_secs().ok()?;
                        let _encoding = self.read_byte().ok()?;
                        let key = self.read_string().ok()?;
                        let value = self.read_string().ok()?;
                        return Some((db, key, value, Some(expiry)));
                    }
                    0xFF => {
                        // End of file
                        return None;
                    }
                    _ => {
                        // Unknown entry type, stop parsing
                        return None;
                    }
                }
            }
        })
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

    fn read_string(&self) -> Result<String, anyhow::Error> {
        let size = self.read_size()?;
        let bytes = self.read_bytes(size)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    fn read_expiry_in_millis(&self) -> Result<u128, anyhow::Error> {
        let bytes = self.read_bytes(8)?;
        let expiry_in_millis = bytes
            .iter()
            .enumerate()
            .fold(0u128, |acc, (i, &b)| acc | ((b as u128) << (i * 8)));
        Ok(expiry_in_millis)
    }

    fn read_expiry_in_secs(&self) -> Result<u128, anyhow::Error> {
        let bytes = self.read_bytes(4)?;
        let expiry_in_secs = bytes
            .iter()
            .enumerate()
            .fold(0u128, |acc, (i, &b)| acc | ((b as u128) << (i * 8)));
        Ok(expiry_in_secs * 1000)
    }
}

#[cfg(test)]
mod specs_for_load {
    use std::io::Write;

    use tempfile::tempfile;

    use super::RdbFileReader;

    #[test]
    fn sut_parses_entries_of_rdb_correctly() {
        // Arrange
        let mut file = tempfile().unwrap();
        file.write_all(header()).unwrap();
        file.write_all(metadata()).unwrap();
        file.write_all(entries()).unwrap();
        file.write_all(footer()).unwrap();

        let sut = RdbFileReader::new(file);

        // Act
        let mut entries = sut.entries();
        let actual = entries.next().unwrap();

        // Assert
        let expected = (0, "foobar".to_string(), "bazqux".to_string(), None);
        assert_eq!(actual, expected);

        // Act
        let actual = entries.next().unwrap();

        // Assert
        let expected = (
            0,
            "foo".to_string(),
            "bar".to_string(),
            Some(1713824559637_u128),
        );
        assert_eq!(actual, expected);

        // Act
        let actual = entries.next().unwrap();

        // Assert
        let expected = (
            0,
            "baz".to_string(),
            "qux".to_string(),
            Some(1714089298000_u128),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn sut_closes_entry_iterator_correctly() {
        // Arrange
        let mut file = tempfile().unwrap();
        file.write_all(header()).unwrap();
        file.write_all(metadata()).unwrap();
        file.write_all(entries()).unwrap();
        file.write_all(footer()).unwrap();

        let sut = RdbFileReader::new(file);
        let mut entries = sut.entries();
        let _entry = entries.next().unwrap();
        let _entry = entries.next().unwrap();
        let _entry = entries.next().unwrap();

        // Act
        let actual = entries.next();

        // Assert
        assert!(actual.is_none());
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

    fn entries() -> &'static [u8] {
        &[
            0xFE, 0x00, 0xFB, 0x01, 0x00, 0x00, 0x06, 0x66, 0x6F, 0x6F, 0x62, 0x61, 0x72, 0x06,
            0x62, 0x61, 0x7A, 0x71, 0x75, 0x78, 0xFC, 0x15, 0x72, 0xE7, 0x07, 0x8F, 0x01, 0x00,
            0x00, 0x00, 0x03, 0x66, 0x6F, 0x6F, 0x03, 0x62, 0x61, 0x72, 0xFD, 0x52, 0xED, 0x2A,
            0x66, 0x00, 0x03, 0x62, 0x61, 0x7A, 0x03, 0x71, 0x75, 0x78,
        ]
    }

    fn footer() -> &'static [u8] {
        &[0xFF, 0x89, 0x3B, 0xB7, 0x4E, 0xF8, 0x0F, 0x77, 0x19]
    }
}
