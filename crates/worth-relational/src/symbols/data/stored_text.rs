//! Physical custody for a symbol's text, independent of semantic string identity.
use super::field_text::RelationalFieldTextAdmissionDenial as Denial;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Borrow,
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use worth_execution::{
    ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial,
};

#[derive(Clone, Debug)]
pub struct StoredSymbolText {
    backing: Backing,
}
#[derive(Clone, Debug)]
enum Backing {
    System(String),
    Managed(ExecutionImmutableBytes),
}
impl StoredSymbolText {
    pub fn as_str(&self) -> &str {
        match &self.backing {
            Backing::System(text) => text,
            Backing::Managed(bytes) => std::str::from_utf8(bytes.bytes()).expect(/* Proof: StoredText is constructed only from str/String or after UTF-8 admission; its private bytes cannot subsequently be mutated. */ "admitted UTF-8"),
        }
    }
    /// Native field-text custody/readmission, which may use explicit System policy.
    /// This predicate never asserts a lease charge; inspect charged_payload_bytes.
    pub fn is_managed(&self) -> bool {
        matches!(self.backing, Backing::Managed(_))
    }
    /// None includes explicit System custody; it is never reported as a lease charge.
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        match &self.backing {
            Backing::System(_) => None,
            Backing::Managed(bytes) => bytes.charged_payload_bytes(),
        }
    }
    pub(crate) fn copy_from_str(
        execution: &ExecutionRequest<'_, '_>,
        text: &str,
    ) -> Result<Self, Denial> {
        execution.check_live().map_err(Denial::Stopped)?;
        let mut bytes = ExecutionRequestArrayBuilder::<u8>::allocate(*execution, text.len())
            .map_err(array_denial)?;
        bytes
            .extend_from_slice(text.as_bytes())
            .map_err(array_denial)?;
        let backing = bytes.seal().map_err(array_denial)?.into_immutable_bytes();
        execution.check_live().map_err(Denial::Stopped)?;
        Ok(Self {
            backing: Backing::Managed(backing),
        })
    }
    pub(crate) fn admit(
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
        Ok(Self {
            backing: Backing::Managed(bytes),
        })
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
            backing: Backing::System(text),
        }
    }
}
impl From<&str> for StoredSymbolText {
    fn from(text: &str) -> Self {
        text.to_owned().into()
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
        match &self.backing {
            Backing::System(text) => serializer.serialize_str(text),
            Backing::Managed(_) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("managed_utf8_v1", self.as_str())?;
                map.end()
            }
        }
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
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            ))
        ));
    }
}
