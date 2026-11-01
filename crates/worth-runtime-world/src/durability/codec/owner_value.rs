//! Stream owned component DTOs without copying their owners' schemas.
use super::{Denial, Emit};
use std::io::{self, Write};

pub(in crate::durability) fn emit<T: serde::Serialize>(
    out: &mut Emit<'_>,
    reference: &T,
) -> Result<(), Denial> {
    let mut refused = None;
    serde_json::to_writer(
        ComponentWriter {
            out,
            refused: &mut refused,
        },
        reference,
    )
    .map_err(|_| refused.unwrap_or(Denial::Malformed))
}
struct ComponentWriter<'a, 'emit> {
    out: &'a mut Emit<'emit>,
    refused: &'a mut Option<Denial>,
}
impl Write for ComponentWriter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match (self.out)(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(cause) => {
                *self.refused = Some(cause);
                Err(io::Error::other("World artifact emission refused"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
