use super::super::{ValueDecodeBounds, ValueSeedDelegate};
use std::any::TypeId;
use std::sync::{Arc, OnceLock};

/// Immutable stored text. The optional codec seal is process-local derived
/// evidence and never crosses serialization or a trust boundary.
#[derive(Clone, Debug)]
pub struct StoredValueText(Box<StoredText>);

#[derive(Debug, Clone)]
struct StoredText {
    text: String,
    seal: Arc<OnceLock<CodecSeal>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodecSeal {
    delegate: TypeId,
    bounds: ValueDecodeBounds,
}

impl StoredValueText {
    /// Untrusted text; the first codec read performs the full canonical check.
    pub fn new(text: String) -> Self {
        Self(Box::new(StoredText {
            text,
            seal: Arc::new(OnceLock::new()),
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
            .saturating_add(self.0.text.capacity())
            .saturating_add(Self::proof_capacity_bytes())
    }

    /// A copy allocates its box and length-sized text. Its proof cell is shared,
    /// so the clone quote excludes that existing allocation and spare capacity.
    pub fn clone_allocation_capacity_bytes(&self) -> usize {
        self.clone_allocation_bytes().unwrap_or(usize::MAX)
    }

    /// Retained text, box and fresh proof-cell backing for a fresh decode.
    /// text_capacity is the decoder's checked String capacity, not wire length.
    /// Text-growth scratch and allocator-private bookkeeping are excluded.
    pub fn fresh_decode_allocation_bytes(text_capacity: usize) -> Option<usize> {
        std::alloc::Layout::array::<u8>(text_capacity).ok()?;
        std::mem::size_of::<StoredText>()
            .checked_add(text_capacity)?
            .checked_add(Self::checked_proof_capacity_bytes()?)
    }

    /// Box and length-sized text only; the existing proof Arc is shared.
    pub fn clone_allocation_bytes(&self) -> Option<usize> {
        std::mem::size_of::<StoredText>().checked_add(self.0.text.len())
    }

    fn checked_proof_capacity_bytes() -> Option<usize> {
        let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<OnceLock<CodecSeal>>())
            .ok()?;
        Some(layout.pad_to_align().size())
    }

    fn proof_capacity_bytes() -> usize {
        Self::checked_proof_capacity_bytes()
            .expect("the fixed codec proof Arc layout fits the address space")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_and_shared_text_quotes_cover_observed_allocations_after_source_drop() {
        let text = "original stored spelling";
        let (source, fresh_peak) =
            crate::allocation_observation::peak(|| StoredValueText::new(text.to_owned()));
        assert_eq!(
            fresh_peak,
            StoredValueText::fresh_decode_allocation_bytes(text.len()).unwrap()
        );
        let clone_bytes = source.clone_allocation_bytes().unwrap();
        let retained = StoredValueText::fresh_decode_allocation_bytes(text.len()).unwrap();
        let (cloned, clone_peak) = crate::allocation_observation::peak(|| source.clone());
        assert_eq!(clone_peak, clone_bytes);
        assert!(
            retained > clone_bytes,
            "shared proof storage is retained, not recopied"
        );
        drop(source);
        assert_eq!(cloned.as_str(), text);
    }

    #[test]
    fn clone_quote_excludes_the_existing_shared_proof_allocation() {
        let source = StoredValueText::new("immutable stored value".to_owned());
        let copy = source.clone();
        assert_ne!(source.as_str().as_ptr(), copy.as_str().as_ptr());
        assert!(Arc::ptr_eq(&source.0.seal, &copy.0.seal));
        assert!(copy.clone_allocation_capacity_bytes() < copy.owned_allocation_capacity_bytes());
    }
}
