use anyhow::Result;
use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::BufReader;
use tokio::io::SeekFrom;
use tokio::sync::Mutex;

use crate::repository::Entry;
use crate::repository::Expiry;
use crate::repository::Repository;
use crate::repository::TimeUnit;

pub async fn load<R: AsyncRead + AsyncSeekExt + Unpin + Send>(
    reader: R,
    repository: Arc<impl Repository>,
) {
    let rdb_file_reader = RdbFileReader::new(reader);
    let mut entries = rdb_file_reader.entries().await;
    while let Some(entry) = entries.next().await {
        if let Some(expiry) = &entry.expiry {
            if expiry.is_expired() {
                continue;
            }
        }
        repository.set(entry).await;
    }
}

struct RdbFileReader<R> {
    reader: Mutex<BufReader<R>>,
}

impl<R: AsyncRead + AsyncSeekExt + Unpin + Send> RdbFileReader<R> {
    pub fn new(reader: R) -> Self {
        RdbFileReader {
            reader: Mutex::new(BufReader::new(reader)),
        }
    }

    async fn initialize(&self) -> Result<()> {
        let mut reader = self.reader.lock().await;
        reader.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    async fn header(&self) -> Result<String> {
        let buffer = self.read_bytes(9).await?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    pub async fn entries(&self) -> Pin<Box<dyn Stream<Item = Entry> + Send + '_>> {
        self.initialize().await.unwrap();
        self.header().await.unwrap();

        Box::pin(stream! {
            loop {
                let entry_type = self.read_byte().await;

                match entry_type {
                    Ok(0xFA) => {
                        // metadata
                        let _key = self.read_string().await;
                        let _value = self.read_string().await;
                        continue;
                    }
                    Ok(0xFE) => {
                        // db selector
                        let _db = self.read_byte().await;
                        continue;
                    }
                    Ok(0xFB) => {
                        // hash table size
                        let _hash_table_size = self.read_size().await;
                        let _expires_hash_table_size = self.read_size().await;
                        continue;
                    }
                    Ok(0x00) => {
                        // entry without expiration
                        if let (Ok(key), Ok(value)) = (self.read_string().await, self.read_string().await) {
                            yield Entry {
                                key,
                                value,
                                expiry: None,
                            };
                        }
                    }
                    Ok(0xFC) => {
                        // entry with milliseconds expiry
                        if let (Ok(expiry_millis), Ok(_encoding), Ok(key), Ok(value)) = (self.read_expiry_in_millis().await, self.read_byte().await, self.read_string().await, self.read_string().await) {
                            yield Entry {
                                key,
                                value,
                                expiry: Some(Expiry {
                                    epoch: expiry_millis,
                                    unit: TimeUnit::Millisecond,
                                }),
                            };
                        }
                    }
                    Ok(0xFD) => {
                        // entry with seconds expiry
                        if let (Ok(expiry_secs), Ok(_encoding), Ok(key), Ok(value)) = (self.read_expiry_in_secs().await, self.read_byte().await, self.read_string().await, self.read_string().await) {
                            yield Entry {
                                key,
                                value,
                                expiry: Some(Expiry {
                                    epoch: expiry_secs,
                                    unit: TimeUnit::Millisecond, // read_expiry_in_secs already converts to milliseconds
                                }),
                            };
                        }
                    }
                    Ok(0xFF) => {
                        // end of file
                        break;
                    }
                    _ => {
                        // unknown entry type, stop parsing
                        break;
                    }
                }
            }
        })
    }

    async fn read_byte(&self) -> Result<u8> {
        let mut buffer = [0u8; 1];
        self.reader.lock().await.read_exact(&mut buffer).await?;
        Ok(buffer[0])
    }

    async fn read_bytes(&self, count: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; count];
        self.reader.lock().await.read_exact(&mut buffer).await?;
        Ok(buffer)
    }

    async fn read_size(&self) -> Result<usize> {
        let first_byte = self.read_byte().await?;
        let first_two_bits = (first_byte >> 6) & 0b11;
        let remaining_bites = first_byte & 0b00111111;
        match first_two_bits {
            0b00 => Ok(remaining_bites as usize),
            0b01 => {
                let second_bytes = self.read_byte().await?;
                Ok(((remaining_bites as usize) << 6) + second_bytes as usize)
            }
            0b10 => {
                let next_four_bytes = self.read_bytes(5).await?;
                Ok(next_four_bytes
                    .iter()
                    .fold(0usize, |acc, &b| (acc << 8) | b as usize))
            }
            0b11 => match remaining_bites {
                0x00 => Ok(1_usize),
                0x01 => Ok(2_usize),
                0x02 => Ok(4_usize),
                _ => unimplemented!(),
            },
            _ => unreachable!(),
        }
    }

    async fn read_string(&self) -> Result<String> {
        let size = self.read_size().await?;
        let bytes = self.read_bytes(size).await?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    async fn read_expiry_in_millis(&self) -> Result<u128> {
        let bytes = self.read_bytes(8).await?;
        let expiry_in_millis = bytes
            .iter()
            .enumerate()
            .fold(0u128, |acc, (i, &b)| acc | ((b as u128) << (i * 8)));
        Ok(expiry_in_millis)
    }

    async fn read_expiry_in_secs(&self) -> Result<u128> {
        let bytes = self.read_bytes(4).await?;
        let expiry_in_secs = bytes
            .iter()
            .enumerate()
            .fold(0u128, |acc, (i, &b)| acc | ((b as u128) << (i * 8)));
        Ok(expiry_in_secs * 1000)
    }
}

#[cfg(test)]
mod specs_for_load {
    use futures::StreamExt;
    use std::io::Cursor;

    use super::RdbFileReader;

    #[tokio::test]
    async fn sut_parses_entries_of_rdb_correctly() {
        // Arrange
        let mut data = Vec::new();
        data.extend_from_slice(sample_rdb());
        let cursor = Cursor::new(data);

        let sut = RdbFileReader::new(cursor);

        // Act
        let entries = sut.entries().await.collect::<Vec<_>>().await;

        // Assert
        insta::assert_debug_snapshot!(entries);
    }

    fn sample_rdb() -> &'static [u8] {
        &[
            // header, REDIS0011 ...............................................................
            0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31,
            // metadata #1, redis-ver: 7.2.0 ...................................................
            0xfa, 0x09, 0x72, 0x65, 0x64, 0x69, 0x73, 0x2d, 0x76, 0x65, 0x72, 0x05, 0x37, 0x2e,
            0x32, 0x2e, 0x30,
            // metadata #2, redis-bits: 64 .....................................................
            0xfa, 0x0a, 0x72, 0x65, 0x64, 0x69, 0x73, 0x2d, 0x62, 0x69, 0x74, 0x73, 0xc0, 0x40,
            // database, 0 .....................................................................
            0xFE, 0x00,
            // hash table sizes ................................................................
            0xFB, 0x01, 0x00,
            // entry #1, foobar: bazqux ........................................................
            0x00, 0x06, 0x66, 0x6F, 0x6F, 0x62, 0x61, 0x72, 0x06, 0x62, 0x61, 0x7A, 0x71, 0x75,
            0x78,
            // entry #2, foo: bar, 1628813948437 milliseconds ..................................
            0xFC, 0x15, 0x72, 0xE7, 0x07, 0x8F, 0x01, 0x00, 0x00, 0x00, 0x03, 0x66, 0x6F, 0x6F,
            0x03, 0x62, 0x61, 0x72,
            // entry #3, baz: qux, 1714006354 seconds ..........................................
            0xFD, 0x52, 0xED, 0x2A, 0x66, 0x00, 0x03, 0x62, 0x61, 0x7A, 0x03, 0x71, 0x75, 0x78,
            // footer ..........................................................................
            0xFF, 0x89, 0x3B, 0xB7, 0x4E, 0xF8, 0x0F, 0x77, 0x19,
        ]
    }
}
