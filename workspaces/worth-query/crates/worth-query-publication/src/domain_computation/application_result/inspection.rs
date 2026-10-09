use super::receipt::WorthQueryPublishedApplicationQueryOmissionPosture;
use super::{
    WorthQueryApplicationQueryPublicationReceipt, WorthQueryPublishedApplicationBasis,
    WorthQueryPublishedApplicationQueryTerminalRelease, WorthQueryPublishedApplicationReadWork,
};

/// Borrowed, non-authoritative view of actual application-query terminal work.
#[derive(Clone, Copy)]
pub struct WorthQueryApplicationQueryPublicationInspection<'receipt> {
    receipt: &'receipt WorthQueryApplicationQueryPublicationReceipt,
}

impl<'receipt> WorthQueryApplicationQueryPublicationInspection<'receipt> {
    pub(super) const fn new(
        receipt: &'receipt WorthQueryApplicationQueryPublicationReceipt,
    ) -> Self {
        Self { receipt }
    }

    pub const fn result_count(&self) -> usize {
        self.receipt.result_count()
    }

    pub fn query_identity(&self) -> &str {
        self.receipt.query_identity()
    }

    pub fn parameter_binding_identity(&self) -> &str {
        self.receipt.parameter_binding_identity()
    }

    pub const fn basis(&self) -> &WorthQueryPublishedApplicationBasis {
        self.receipt.basis()
    }

    pub const fn ordinary_work_units(&self) -> usize {
        self.receipt.ordinary_work_units()
    }

    /// Actual lower read counters, separate from authorization and publication.
    pub const fn read_work(&self) -> WorthQueryPublishedApplicationReadWork {
        self.receipt.read_work()
    }

    /// Ordinary authorization observation work. This excludes host preparation.
    pub const fn authorization_work_units(&self) -> usize {
        self.receipt.authorization_work_units()
    }

    pub const fn omission_posture(&self) -> WorthQueryPublishedApplicationQueryOmissionPosture {
        self.receipt.omission_posture()
    }

    pub const fn publication_canonical_entries(&self) -> u32 {
        self.receipt.publication_work().canonical_entries()
    }

    pub const fn publication_sha256_compression_blocks(&self) -> usize {
        self.receipt.publication_work().sha256_compression_blocks()
    }

    pub const fn publication_identity_text_materializations(&self) -> u32 {
        self.receipt
            .publication_work()
            .digest_text_materializations()
    }

    pub const fn terminal_resources_released(&self) -> bool {
        self.receipt.terminal_release().resources_released()
    }

    pub const fn terminal_release(&self) -> WorthQueryPublishedApplicationQueryTerminalRelease {
        self.receipt.terminal_release()
    }
}
