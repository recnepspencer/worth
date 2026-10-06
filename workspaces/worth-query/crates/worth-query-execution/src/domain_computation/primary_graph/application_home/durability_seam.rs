//! Image bytes for durability tests: tampering, a fresh process, one state opened twice.
//!
//! The seam only builds a home value. `open` stays the single path to a runtime.

use super::{ApplicationHome, HomeForm};
use crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint;

impl ApplicationHome {
    /// The closed image's bytes, or `None` when this home holds no image.
    #[doc(hidden)]
    pub fn image_bytes_for_durability_test(&self) -> Option<&[u8]> {
        match &self.0 {
            HomeForm::Memory(image) => image.as_ref().map(WorthQueryApplicationCheckpoint::bytes),
            HomeForm::At(_) => None,
        }
    }

    /// A memory home holding untrusted image bytes; open still validates them.
    #[doc(hidden)]
    pub fn memory_from_image_bytes_for_durability_test(bytes: Vec<u8>) -> Self {
        Self::holding(WorthQueryApplicationCheckpoint::from_untrusted_bytes(
            bytes.into_boxed_slice(),
        ))
    }

    /// A memory home whose image body was rewritten and re-authenticated, or
    /// `None` when this home holds no image.
    #[doc(hidden)]
    pub fn rewrite_authenticated_body_for_durability_test(
        &self,
        rewrite: impl FnOnce(&mut [u8]),
    ) -> Option<Self> {
        match &self.0 {
            HomeForm::Memory(image) => image
                .as_ref()
                .map(|image| image.rewrite_authenticated_body_for_durability_test(rewrite))
                .map(Self::holding),
            HomeForm::At(_) => None,
        }
    }
}
