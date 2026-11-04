//! Encode the two fixed current-ownership scalar fields under original funding.
use super::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

pub(crate) fn encode(
    aspect: &str,
    publication: CommitId,
    bytes: Vec<u8>,
) -> Result<AspectFieldPatch, TransactionCommitError> {
    let text = String::from_utf8(bytes)
        .map_err(|_| crate::mvcc::origin_denial("Native ownership payload UTF-8 refused"))?;
    Ok(AspectFieldPatch::new(BTreeMap::from([
        (
            locator(aspect, "publication"),
            AspectValue::UInt64(publication.0),
        ),
        (locator(aspect, "payload"), AspectValue::String(text.into())),
    ])))
}
impl Writer<'_, '_, '_, '_, '_> {
    pub(crate) fn encode<T: Serialize>(
        &mut self,
        value: &T,
    ) -> Result<Vec<u8>, TransactionCommitError> {
        let (width, stop, result) = {
            let mut counter = Width {
                bytes: 0,
                charge: |units| self.checkpoint(units),
                stop: None,
            };
            let result = serde_json::to_writer(&mut counter, value);
            (counter.bytes, counter.stop, result)
        };
        if let Some(stop) = stop {
            return Err(stop);
        }
        result.map_err(|_| crate::mvcc::origin_denial("Native ownership payload width refused"))?;
        // The exact-sized Vec and its Raw string carrier coexist during handoff.
        self.retain((width as u64).checked_mul(2).ok_or_else(|| {
            request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?)?;
        let mut bytes = Vec::with_capacity(width);
        serde_json::to_writer(&mut bytes, value)
            .map_err(|_| crate::mvcc::origin_denial("Native ownership payload encoding refused"))?;
        Ok(bytes)
    }
}
struct Width<F> {
    bytes: usize,
    charge: F,
    stop: Option<TransactionCommitError>,
}
impl<F: FnMut(usize) -> Result<(), TransactionCommitError>> Write for Width<F> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let result = bytes
            .len()
            .checked_mul(2)
            .ok_or_else(|| {
                request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow)
            })
            .and_then(|units| (self.charge)(units));
        if let Err(stop) = result {
            self.stop = Some(stop);
            return Err(std::io::Error::other("ownership payload width stopped"));
        }
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("ownership payload width overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).expect("fixed Native current aspect"),
        CanonicalFieldPath::single(FieldKey::new(field).expect("fixed Native current field")),
    )
}
