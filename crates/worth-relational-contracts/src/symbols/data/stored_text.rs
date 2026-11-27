//! Physical custody for a symbol's text, independent of semantic string identity.
pub mod index_key;
use super::field_text::RelationalFieldTextAdmissionDenial as Denial;
pub use index_key::SymbolTextLookupKey;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};
use worth_execution::{
    ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial,
};

/// One immutable, reference-counted byte owner. External backing is explicit
/// System custody (never charged); execution-allocated backing is Managed
/// custody (charged, or admitted under an explicit System policy). Clones share
/// the bytes: a persistent dictionary copies every sibling entry of a node on a
/// path copy, so a deep-copying carrier would make each commit pay for the
/// client keys already in the dictionary (public #446).
#[derive(Clone)]
pub struct StoredSymbolText {
    bytes: ExecutionImmutableBytes,
}
// A shared handle like SymbolTextLookupKey, never an inline owned String.
const _: () = assert!(std::mem::size_of::<StoredSymbolText>() <= 2 * std::mem::size_of::<usize>());

impl StoredSymbolText {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.bytes.bytes()).expect(/* Proof: StoredText is constructed only from str/String or after UTF-8 admission; its private bytes cannot subsequently be mutated. */ "admitted UTF-8")
    }
    /// Native field-text custody/readmission, which may use explicit System policy.
    /// This predicate never asserts a lease charge; inspect charged_payload_bytes.
    pub fn is_managed(&self) -> bool {
        !self.bytes.is_external_backing()
    }
    /// None includes explicit System custody; it is never reported as a lease charge.
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.bytes.charged_payload_bytes()
    }
    /// Whether `other` is this exact custody: equal System text, or the same
    /// Managed allocation. Replacing one with the other changes nothing.
    pub fn is_same_custody(&self, other: &Self) -> bool {
        match (self.is_managed(), other.is_managed()) {
            (false, false) => self.as_str() == other.as_str(),
            (true, true) => {
                !self.as_str().is_empty() && std::ptr::eq(self.as_str(), other.as_str())
            }
            (false, true) | (true, false) => false,
        }
    }
}
impl std::fmt::Debug for StoredSymbolText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredSymbolText")
            .field("text", &self.as_str())
            .field("managed", &self.is_managed())
            .finish()
    }
}
fn array_denial(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => Denial::Request(cause.into()),
        ExecutionRequestArrayDenial::Allocation(cause) => Denial::Allocation(cause),
    }
}
impl From<String> for StoredSymbolText {
    fn from(text: String) -> Self {
        Self {
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(
                text.into_bytes().into_boxed_slice(),
            )),
        }
    }
}
impl From<&str> for StoredSymbolText {
    fn from(text: &str) -> Self {
        Self {
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(Box::from(
                text.as_bytes(),
            ))),
        }
    }
}
impl Borrow<str> for StoredSymbolText {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
impl AsRef<str> for StoredSymbolText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl PartialEq for StoredSymbolText {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for StoredSymbolText {}
impl PartialOrd for StoredSymbolText {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for StoredSymbolText {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl Hash for StoredSymbolText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
impl PartialEq<str> for StoredSymbolText {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<&str> for StoredSymbolText {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<String> for StoredSymbolText {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}
impl Serialize for StoredSymbolText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.is_managed() {
            return serializer.serialize_str(self.as_str());
        }
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("managed_utf8_v1", self.as_str())?;
        map.end()
    }
}
impl<'de> Deserialize<'de> for StoredSymbolText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Context-free configuration/import can read System text only. Managed
        // maps refuse before visiting their text, never allocating a hidden DTO.
        struct System;
        impl<'de> serde::de::Visitor<'de> for System {
            type Value = StoredSymbolText;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("System text; managed text requires native execution readmission")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(value.into())
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(value.into())
            }
        }
        deserializer.deserialize_any(System)
    }
}

/// Operations used by the Relational owners.
pub trait StoredSymbolTextOwnerPort: Sized {
    fn copy_from_str(execution: &ExecutionRequest<'_, '_>, text: &str) -> Result<Self, Denial>;
    fn admit(
        execution: &ExecutionRequest<'_, '_>,
        text: &ExecutionImmutableBytes,
    ) -> Result<Self, Denial>;
}

impl StoredSymbolTextOwnerPort for StoredSymbolText {
    fn copy_from_str(execution: &ExecutionRequest<'_, '_>, text: &str) -> Result<Self, Denial> {
        execution.check_live().map_err(Denial::Stopped)?;
        let mut bytes = ExecutionRequestArrayBuilder::<u8>::allocate(*execution, text.len())
            .map_err(array_denial)?;
        bytes
            .extend_from_slice(text.as_bytes())
            .map_err(array_denial)?;
        let bytes = bytes.seal().map_err(array_denial)?.into_immutable_bytes();
        execution.check_live().map_err(Denial::Stopped)?;
        Ok(Self { bytes })
    }

    fn admit(
        execution: &ExecutionRequest<'_, '_>,
        text: &ExecutionImmutableBytes,
    ) -> Result<Self, Denial> {
        execution.check_live().map_err(Denial::Stopped)?;
        std::str::from_utf8(text.bytes()).map_err(|_| Denial::InvalidUtf8)?;
        let bytes = if text.charged_payload_bytes().is_some() {
            text.clone()
        } else {
            // None is not admission: import through this SAME request rather
            // than allocation_policy(), which is System for a serial request.
            return Self::copy_from_str(
                execution,
                std::str::from_utf8(text.bytes()).map_err(|_| Denial::InvalidUtf8)?,
            );
        };
        execution.check_live().map_err(Denial::Stopped)?;
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

    #[test]
    fn owned_decode_original_serial_text_admission_retains_exact_bytes_through_clone() {
        let cancellation = CancellationSource::new();
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(2 << 20)),
            cancellation.token(),
            None,
        );
        let request = ExecutionRequest::serial(&serial);
        let original = "actual retained UTF-8: \u{03bb}";
        let text = StoredSymbolText::copy_from_str(&request, original).unwrap();
        let expected = original.len() as u64;
        assert_eq!(text.charged_payload_bytes(), Some(expected));
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), expected);
        let retained = text.clone();
        drop(text);
        assert_eq!(retained.as_str(), original);
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), expected);
        drop(retained);
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
        cancellation.cancel();
        assert!(matches!(
            StoredSymbolText::copy_from_str(&request, "must refuse"),
            Err(Denial::Stopped(_))
        ));
    }

    #[test]
    fn clones_and_lookup_keys_share_the_bytes_and_keep_custody() {
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(2 << 20)),
            CancellationSource::new().token(),
            None,
        );
        let request = ExecutionRequest::serial(&serial);
        let system = StoredSymbolText::from("system text");
        let managed = StoredSymbolText::copy_from_str(&request, "managed text").unwrap();
        for text in [system, managed] {
            let keyed = SymbolTextLookupKey::from_text(&text).retained_text();
            for copy in [text.clone(), keyed] {
                assert!(std::ptr::eq(copy.as_str(), text.as_str()));
                assert_eq!(copy.is_managed(), text.is_managed());
                assert_eq!(copy.charged_payload_bytes(), text.charged_payload_bytes());
                assert!(copy.is_same_custody(&text));
            }
        }
    }

    #[test]
    fn owned_decode_external_text_import_uses_original_serial_budget_and_keeps_denial() {
        let cancellation = CancellationSource::new();
        let external = ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(
            b"original external".to_vec().into_boxed_slice(),
        ));
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(2 << 20)),
            cancellation.token(),
            None,
        );
        let request = ExecutionRequest::serial(&serial);
        let imported = StoredSymbolText::admit(&request, &external).unwrap();
        assert_eq!(imported.as_str().as_bytes(), external.bytes());
        assert_eq!(
            imported.charged_payload_bytes(),
            Some(external.bytes().len() as u64)
        );
        drop(imported);
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
        let exhausted = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(0)),
            cancellation.token(),
            None,
        );
        assert!(matches!(
            StoredSymbolText::admit(&ExecutionRequest::serial(&exhausted), &external),
            Err(Denial::Request(
                crate::execution::denial_cause::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            ))
        ));
    }
}
