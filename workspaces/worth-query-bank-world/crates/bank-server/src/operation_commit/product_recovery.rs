//! Bank lanes over the installed owner's unpublished product recovery.
//!
//! A commit whose durable product settled but whose publication did not is
//! recovered through the owner, never through a caller-held program runtime.

use std::num::NonZeroUsize;

use worth_query_host::facade::primary_graph::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryCursor, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial, WorthQueryProductUnpublishedRecovery,
};

use crate::BankIdentityRuntime;

impl BankIdentityRuntime {
    /// Lists, in stable order, commits whose product awaits publication.
    ///
    /// # Errors
    ///
    /// Returns the owner's denial when the recovery page cannot be read.
    pub fn product_publication_recovery_page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
    {
        self.application_program()
            .runtime()
            .product_publication_recovery_page(after, maximum)
    }

    /// Readmits one listed commit so its product publishes through the owner.
    ///
    /// # Errors
    ///
    /// Returns the owner's denial when the handle no longer names a
    /// recoverable commit.
    pub fn readmit_product_publication_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductUnpublishedRecovery, RuntimeWorldRecoveryDenial> {
        self.application_program()
            .runtime()
            .readmit_product_publication_recovery(handle)
    }
}
