//! Transport Native's owned serde grammar without copying its component schema.
use super::{Denial, Emit};
use std::io::{self, Write};
use worth_relational::facade::branch::RecordedRelationalComponentReference;

pub(super) fn emit(
    out: &mut Emit<'_>,
    reference: &RecordedRelationalComponentReference,
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
