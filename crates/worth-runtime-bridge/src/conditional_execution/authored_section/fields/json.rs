//! Streaming pure semantic fields avoids an unbilled temporary JSON buffer.
use super::{Writer, D};

struct JsonSink<'a> {
    emit: &'a mut dyn FnMut(&[u8]) -> Result<(), D>,
    refused: Option<D>,
}
impl std::io::Write for JsonSink<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match (self.emit)(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(refused) => {
                self.refused = Some(refused);
                Err(std::io::Error::other(
                    "Bridge semantic field emission refused",
                ))
            }
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn emit<T: serde::Serialize>(
    value: &T,
    target: &mut dyn FnMut(&[u8]) -> Result<(), D>,
) -> Result<(), D> {
    let mut sink = JsonSink {
        emit: target,
        refused: None,
    };
    serde_json::to_writer(&mut sink, value)
        .map_err(|_| sink.refused.take().unwrap_or(D::SemanticContractEncoding))
}
pub(super) fn encode<T: serde::Serialize>(out: &mut Writer<'_>, value: &T) -> Result<(), D> {
    let mut size = 0usize;
    emit(value, &mut |bytes| {
        out.charge_bytes(bytes.len())?;
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= super::super::framing::MAXIMUM_BYTES)
            .ok_or(D::SizeLimit)?;
        Ok(())
    })?;
    out.number(size as u64)?;
    emit(value, &mut |bytes| out.fixed(bytes))
}
