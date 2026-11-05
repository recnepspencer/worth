//! Quote private payload backing before deserializing the closed Native grammar.
use super::*;
use serde::de::{DeserializeOwned, DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserializer, Serialize};
use std::fmt;

pub(crate) fn decode<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    publication: CommitId,
    record: &RecordRef,
    funding: &mut ExecutionMemoryReservation,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<T, TransactionCommitError> {
    let bad = || malformed(publication, record);
    // serde_json may grow its escaped-string scratch to twice the payload width.
    // Fund it before either pass; the quote separately covers retained values.
    grow(
        funding,
        (bytes.len() as u64).checked_mul(2).ok_or_else(|| {
            super::super::request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            )
        })?,
    )?;
    let mut quote = Quote {
        // Publication is the only payload with an Arc-owned origin. Counting
        // it for the smaller catalog/edge payloads is a conservative fixed quote.
        bytes: (std::mem::size_of::<T>()
            + std::mem::size_of::<crate::history::data::CanonicalRequiredOutputOrigin>()
            + 2 * std::mem::size_of::<usize>()) as u64,
        context,
        stop: None,
    };
    checkpoint(
        quote.context,
        (bytes.len() as u64).checked_mul(3).ok_or_else(|| {
            super::super::request(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        })?,
    )?;
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let result = Probe {
        quote: &mut quote,
        element: 0,
    }
    .deserialize(&mut decoder);
    // The value pass reuses this funded scratch extent, rather than retaining
    // the quote pass's decoder alongside a second scratch allocation.
    drop(decoder);
    if let Some(stop) = quote.stop {
        return Err(stop);
    }
    result.map_err(|_| bad())?;
    grow(funding, quote.bytes)?;
    let value: T = serde_json::from_slice(bytes).map_err(|_| bad())?;
    let mut canonical = CanonicalBytes(bytes);
    if serde_json::to_writer(&mut canonical, &value).is_err() || !canonical.0.is_empty() {
        return Err(bad());
    }
    Ok(value)
}

pub(super) fn malformed(publication: CommitId, record: &RecordRef) -> TransactionCommitError {
    if publication == CommitId(0) {
        damaged(record)
    } else {
        corrupt(publication, record.clone())
    }
}

struct Quote<'a, 'scope, 'authority> {
    bytes: u64,
    context: &'a mut MapKernelContext<'scope, 'authority>,
    stop: Option<TransactionCommitError>,
}
impl Quote<'_, '_, '_> {
    fn add<E: Error>(&mut self, bytes: usize) -> Result<(), E> {
        let result = checkpoint(self.context, 1).and_then(|()| {
            self.bytes.checked_add(bytes as u64).ok_or_else(|| {
                super::super::request(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })
        });
        match result {
            Ok(bytes) => {
                self.bytes = bytes;
                Ok(())
            }
            Err(stop) => {
                self.stop = Some(stop);
                Err(E::custom("current ownership payload quote stopped"))
            }
        }
    }
}
struct Probe<'a, 'b, 'scope, 'authority> {
    quote: &'a mut Quote<'b, 'scope, 'authority>,
    element: usize,
}
impl<'de> DeserializeSeed<'de> for Probe<'_, '_, '_, '_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        self.quote.add::<D::Error>(0)?;
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Probe<'_, '_, '_, '_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Native ownership record")
    }
    fn visit_unit<E: Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_none<E: Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_bool<E: Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: Error>(self, value: &str) -> Result<(), E> {
        self.quote.add(value.len())
    }
    fn visit_borrowed_str<E: Error>(self, value: &'de str) -> Result<(), E> {
        self.visit_str(value)
    }
    fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<(), E> {
        self.quote.add(value.len())
    }
    fn visit_borrowed_bytes<E: Error>(self, value: &'de [u8]) -> Result<(), E> {
        self.visit_bytes(value)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while let Some(()) = seq.next_element_seed(Probe {
            quote: self.quote,
            element: 0,
        })? {
            self.quote.add(
                self.element
                    .checked_mul(4)
                    .ok_or_else(|| A::Error::custom("ownership sequence quote overflow"))?,
            )?;
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        while let Some(key) = map.next_key::<&str>()? {
            use crate::history::data::{CanonicalOutputMember, CanonicalRequiredOutputRelation};
            // These are all growable sequences in this closed grammar; digest
            // arrays and identifier tuples are inline and carry zero backing.
            let element = match key {
                "members" => std::mem::size_of::<CanonicalOutputMember>(),
                "relations" => std::mem::size_of::<CanonicalRequiredOutputRelation>(),
                "outputs" => std::mem::size_of::<EntityId>(),
                "routes" => std::mem::size_of::<record::Route>(),
                "parents" | "parent_commit_ids" => std::mem::size_of::<CommitId>(),
                _ => 0,
            };
            map.next_value_seed(Probe {
                quote: self.quote,
                element,
            })?;
        }
        Ok(())
    }
}

// Compare the canonical serialization in place, without an unquoted second Vec.
struct CanonicalBytes<'a>(&'a [u8]);
impl std::io::Write for CanonicalBytes<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if !self.0.starts_with(bytes) {
            return Err(std::io::Error::other("noncanonical ownership payload"));
        }
        self.0 = &self.0[bytes.len()..];
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn grow(
    funding: &mut ExecutionMemoryReservation,
    bytes: u64,
) -> Result<(), TransactionCommitError> {
    let extent = funding.bytes().checked_add(bytes).ok_or_else(|| {
        super::super::request(
            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
        )
    })?;
    funding.resize(extent).map_err(|cause| {
        super::super::request(crate::execution::denial::lease_denial(
            worth_execution::LeaseDenial::MemoryExhausted(cause),
        ))
    })
}
