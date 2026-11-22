use std::sync::Arc;

use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDisposition, WorthQueryArtifactHandleGuard,
    WorthQueryArtifactSemanticProjection, WorthQueryRuntimeArtifactOwner,
};

pub struct WorthQueryBorrowedArtifactView<'a> {
    owner: &'a Arc<WorthQueryRuntimeArtifactOwner>,
    borrow_generation: u64,
    purpose: String,
}

impl<'a> WorthQueryBorrowedArtifactView<'a> {
    pub(super) fn admit(
        owner: &'a Arc<WorthQueryRuntimeArtifactOwner>,
        guard: WorthQueryArtifactHandleGuard,
        purpose: impl Into<String>,
    ) -> Result<Self, WorthQueryArtifactDenial> {
        let borrow_generation = owner.admit_borrow(guard)?;
        Ok(Self {
            owner,
            borrow_generation,
            purpose: purpose.into(),
        })
    }

    pub fn semantic_projection(&self) -> &WorthQueryArtifactSemanticProjection {
        self.owner.semantic_projection()
    }

    pub const fn borrow_generation(&self) -> u64 {
        self.borrow_generation
    }

    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    pub fn disposition(&self) -> WorthQueryArtifactDisposition {
        WorthQueryArtifactDisposition::Borrowed
    }
}

impl Drop for WorthQueryBorrowedArtifactView<'_> {
    fn drop(&mut self) {
        self.owner
            .release_borrow(self.borrow_generation)
            .expect("borrowed artifact view releases exactly one active generation");
    }
}
