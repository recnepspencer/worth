//! Bind each owner-facing read to installed Native identity and canonical history.
use super::RelationalRuntime;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::mvcc::publication::application_publication::{
    RelationalInstalledApplicationPublicationAuthority, RelationalRequiredOutputPublication,
    RelationalRequiredOutputRouteAbsence, RelationalRestoredProducerPrior,
    RequiredOutputPublicationMeaning,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Funded cold admission when the recovered selector does not cover its
    /// canonical prefix. A complete warm image uses only its reached route.
    /// `None` means the route has a publication and requires exact predecessor
    /// custody; it is never permission to begin another ownership birth.
    pub fn readmit_required_output_route_absence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputRouteAbsence>, TransactionCommitError> {
        crate::mvcc::publication::application_publication::route_absence::readmit_required_output_route_absence(self.into(), execution, installed, basis, meaning)
    }

    /// Explicit cold readmission for a restored correspondence. Warm callers
    /// retain the prior native holder and never traverse canonical history.
    pub fn readmit_required_output_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
        producer: &str,
        dependency: &[u8; 32],
        meaning: &crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
        roles: &[crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget],
    ) -> Result<
        RelationalRequiredOutputPublication,
        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        crate::mvcc::publication::application_publication::custody::readmit_required_output_publication(self.into(), execution, installed, basis, producer, dependency, meaning, roles)
    }

    pub fn readmit_current_owned_output_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        running_producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        crate::mvcc::publication::application_publication::custody::current::readmit_successor(
            self.runtime_instance_id(),
            execution,
            installed,
            basis,
            running_producer,
            meaning,
        )
    }

    /// Renew a basis-bound holder through the existing Native custody owner.
    /// Canonical carried origins retain their ordinary predecessor validation.
    pub fn readmit_carried_required_output_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        carried: &RelationalRequiredOutputPublication,
    ) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
        crate::mvcc::publication::application_publication::native_transition_succession::readmit_carried_required_output_publication(self.into(), execution, installed, basis, carried)
    }

    /// Reissue succeeds the actual Native route transition, never a fresh Birth.
    /// Active owned membership is authenticated by the current-custody owner.
    pub fn readmit_native_output_transition_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        crate::mvcc::publication::application_publication::native_transition_succession::readmit_native_output_transition_successor(self.into(), execution, installed, basis, meaning)
    }

    pub(crate) fn readmit_canonical_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        origin: &Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    ) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
        crate::mvcc::publication::application_publication::native_transition_succession::readmit_canonical_publication(self.into(), execution, installed, basis, origin)
    }

    /// Reissue an independent route's original publication custody at current C.
    /// Payload currentness belongs to the caller's verification; native successor
    /// installation still checks the full predecessor and relation correspondence.
    pub fn readmit_current_independent_output_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        running_producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        crate::mvcc::publication::application_publication::native_transition_succession::independent::readmit_current_independent_output_successor(self.into(), execution, installed, basis, running_producer, meaning)
    }

    pub fn readmit_restored_producer_prior(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRestoredProducerPrior>, TransactionCommitError> {
        crate::mvcc::publication::application_publication::restored_producer_prior::readmit_restored_producer_prior(self.into(), execution, installed, basis, meaning)
    }
}
