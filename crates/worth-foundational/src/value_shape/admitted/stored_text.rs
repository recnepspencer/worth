use super::super::{ValueDecodeBounds, ValueSeedDelegate};
use std::any::TypeId;
use std::sync::{Arc, OnceLock};

/// Immutable stored text. The optional codec seal is process-local derived
/// evidence and never crosses serialization or a trust boundary.
#[derive(Clone, Debug)]
pub struct StoredValueText(Arc<StoredText>);

#[derive(Debug)]
struct StoredText {
    text: String,
    seal: OnceLock<CodecSeal>,
}

#[derive(Debug, PartialEq, Eq)]
struct CodecSeal {
    delegate: TypeId,
    bounds: ValueDecodeBounds,
}

impl StoredValueText {
    /// Untrusted text; the first codec read performs the full canonical check.
    pub fn new(text: String) -> Self {
        Self(Arc::new(StoredText {
            text,
            seal: OnceLock::new(),
        }))
    }

    pub fn is_empty(&self) -> bool {
        self.0.text.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.0.text
    }
    pub fn as_string(&self) -> &String {
        &self.0.text
    }
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        std::mem::size_of::<StoredText>()
            .saturating_add(2 * std::mem::size_of::<usize>())
            .saturating_add(self.0.text.capacity())
    }

    pub(in crate::value_shape) fn matches<Delegate: ValueSeedDelegate>(
        &self,
        bounds: ValueDecodeBounds,
    ) -> bool {
        self.0.seal.get()
            == Some(&CodecSeal {
                delegate: TypeId::of::<Delegate>(),
                bounds,
            })
    }

    pub(in crate::value_shape) fn seal<Delegate: ValueSeedDelegate>(
        &self,
        bounds: ValueDecodeBounds,
    ) {
        // A different codec/policy always receives the full check. Never replace
        // evidence or infer validity for a different grammar from equal text.
        let _ = self.0.seal.set(CodecSeal {
            delegate: TypeId::of::<Delegate>(),
            bounds,
        });
    }
}

impl serde::Serialize for StoredValueText {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
