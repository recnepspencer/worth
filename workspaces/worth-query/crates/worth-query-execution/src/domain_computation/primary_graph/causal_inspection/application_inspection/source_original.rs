//! A performed source shares only immutable canonical/disclosure custody.
use super::admission::InspectionAdmissionDenial as Denial;
use crate::domain_computation::primary_graph::{
    source_disclosure::OriginalSourceInspectionDisclosure, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommittedChanges,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::{
    ApplicationSourceDisclosureBinding, DeclaredApplicationSourceDisclosure,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationQueryBinding,
};

#[derive(Clone)]
pub(super) struct SourceOriginal {
    pub(super) runtime: u64,
    pub(super) operation: [u8; 32],
    pub(super) publication: crate::basis::WorthQueryProductBranchReadIdentity,
    pub(super) changes: WorthQueryApplicationCommittedChanges,
    pub(super) disclosure: Arc<OriginalSourceInspectionDisclosure>,
}
impl SourceOriginal {
    /// Only allocation-free structural affinity precedes fresh READ/metadata.
    /// No patch, field, count or parameter value is traversed here.
    pub(super) fn from_receipt<Schema, Disclosure>(
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        installed: &WorthQueryInstalledApplicationQueryBinding<Schema, Disclosure::Disclosure>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Self, Denial>
    where
        Schema: ApplicationSchema,
        Disclosure: ApplicationSourceDisclosureBinding<Schema>,
    {
        let authority = receipt.authority_binding();
        if authority.runtime_authority() != runtime.runtime.authority_identity()
            || authority.original_mutation_binding()
                != Some(std::any::TypeId::of::<Disclosure::Source>())
        {
            return Err(Denial::ForeignIssuedAccess);
        }
        let disclosure = authority
            .retain_original_inspection_disclosure()
            .ok_or(Denial::MissingInspection)?;
        if disclosure.association().declaration()
            != &DeclaredApplicationSourceDisclosure::of::<Schema, Disclosure>()
            || disclosure.association().source_type() != std::any::TypeId::of::<Disclosure::Source>()
            || disclosure.association().query_type() != std::any::TypeId::of::<<Disclosure::Disclosure as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query>()
            || disclosure.schema() != installed.query().binding_identity()
            || disclosure.disclosure().query() != installed.query().identity()
        {
            return Err(Denial::ForeignIssuedAccess);
        }
        let publication = receipt
            .committed_product_publication()
            .observed_product_identity()
            .ok_or(Denial::MissingInspection)?;
        Ok(Self {
            runtime: authority.runtime_authority().as_u64(),
            operation: *authority.installed_operation(),
            publication,
            changes: receipt.committed_changes().clone(),
            disclosure,
        })
    }
    pub(super) fn same_original(&self, other: &Self) -> bool {
        self.runtime == other.runtime
            && self.operation == other.operation
            && self.publication == other.publication
            && self.changes.shares_original(&other.changes)
            && Arc::ptr_eq(&self.disclosure, &other.disclosure)
    }
}
