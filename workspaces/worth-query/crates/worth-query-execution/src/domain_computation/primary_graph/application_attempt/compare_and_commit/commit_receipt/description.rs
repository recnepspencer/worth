//! A receipt can retain its original evidence without retaining physical history.
use super::WorthQueryApplicationCommitReceipt;

impl WorthQueryApplicationCommitReceipt {
    /// Copy original immutable receipt evidence without a performed-change
    /// witness, retained product basis or World history protection. This does
    /// not retire the original publication or release any sibling's custody.
    /// Ordinary `Clone` continues to retain its original publication custody.
    pub fn descriptive_copy(&self) -> Self {
        let mut copy = self.clone();
        copy.inner.committed_product_publication =
            self.inner.committed_product_publication.descriptive();
        copy
    }
}
