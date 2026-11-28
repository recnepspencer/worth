//! Hash serializer output without retaining a second copy of its bytes.
use sha2::{Digest, Sha256};
use std::io::{self, Write};

#[derive(Default)]
pub struct Sha256Sink(pub Sha256);

impl Write for Sha256Sink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Meter each original serializer write in bounded chunks before hashing.
pub struct MeteredSha256Sink<'a, E> {
    hash: Sha256Sink,
    charge: &'a mut dyn FnMut(usize) -> Result<(), E>,
    failure: Option<E>,
    bytes: u64,
}

impl<'a, E> MeteredSha256Sink<'a, E> {
    pub fn new(hash: Sha256, charge: &'a mut dyn FnMut(usize) -> Result<(), E>) -> Self {
        Self {
            hash: Sha256Sink(hash),
            charge,
            failure: None,
            bytes: 0,
        }
    }

    pub fn failure(&self) -> Option<&E> {
        self.failure.as_ref()
    }

    pub fn into_parts(self) -> (Sha256, u64, Option<E>) {
        (self.hash.0, self.bytes, self.failure)
    }
}

impl<E> Write for MeteredSha256Sink<'_, E> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::ErrorKind::Other.into());
        }
        for chunk in input.chunks(64 * 1024) {
            if let Err(cause) = (self.charge)(chunk.len()) {
                self.failure = Some(cause);
                return Err(io::ErrorKind::Other.into());
            }
            self.bytes = self
                .bytes
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| io::Error::other("canonical digest length overflow"))?;
            self.hash.write_all(chunk)?;
        }
        Ok(input.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
