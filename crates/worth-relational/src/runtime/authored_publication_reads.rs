//! Installed Runtime receivers bind authored reads to the actual history owner.
use super::RelationalRuntime;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::mvcc::publication::application_publication::{
    AuthoredOwnedMemberKind, AuthoredPublicationReads, RelationalAuthoredCoupledOwners,
    RelationalAuthoredRetirement, RelationalInstalledApplicationPublicationAuthority,
    RelationalRequiredOutputPublication, RequiredOutputRoleTarget,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::CreatedEntityRef;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Native computes the complete opted-in source-scope closure before judging
    /// the submitted roster. The anchor is a distinguished authored source.
    pub fn readmit_authored_source_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<Option<RelationalAuthoredRetirement>, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_authored_source_retirement(execution, installed, basis, anchor, roles)
    }

    /// Each explicitly declared co-anchor is an independently authenticated authored source.
    pub fn readmit_authored_source_retirement_with_co_anchors(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        co_anchors: &[EntityId],
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<Option<RelationalAuthoredRetirement>, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_authored_source_retirement_with_co_anchors(
                execution, installed, basis, anchor, co_anchors, roles,
            )
    }

    /// The declared member kinds opt only this anchor into roster-only history.
    /// Native certifies the exact owner adjacency before minting that event.
    pub fn readmit_authored_owned_member_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        anchors: (EntityId, &[EntityId]),
        roles: &[RequiredOutputRoleTarget],
        kinds: &[AuthoredOwnedMemberKind],
    ) -> Result<Option<RelationalAuthoredRetirement>, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into()).readmit_authored_owned_member_retirement(
            execution, installed, basis, anchors, roles, kinds,
        )
    }

    /// Reconstruct the selected complete publication behind real ordinary Retire
    /// targets. No matching canonical ownership leaves genuine ordinary deletion
    /// ordinary; any matched ownership requires its complete exact roster.
    pub fn readmit_authored_output_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_authored_output_retirement(execution, installed, basis, roles)
    }

    /// Mixed ordinary candidates retain legacy behavior for independent outputs,
    /// but cannot hide an owned member or its live source beside Create/Preserve roles.
    pub fn refuse_owned_output_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<(), TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .refuse_owned_output_retirement(execution, installed, basis, roles)
    }

    /// O((A + R) K + A log A + R log R) over incoming owner adjacency A,
    /// submitted member roles R and declared member kinds K. Membership has no second authoritative list.
    /// A request with no work account is polled, not charged.
    pub fn validate_authored_owned_members(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        kinds: &[AuthoredOwnedMemberKind],
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<(), TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .validate_authored_owned_members(execution, basis, anchor, kinds, roles)
    }

    /// The closure owner authenticates all anchors and current publications once.
    /// Surviving edge consumers are readmitted through the existing custody owner.
    pub fn readmit_authored_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        co_anchors: &[EntityId],
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_authored_coupled_owners(execution, installed, basis, anchor, co_anchors, roles)
    }

    /// Retain correspondence for the actual corrective prefix. The caller has
    /// already authenticated its original correction meaning. This data grants
    /// no retirement/publication authority and selects no arbitrary draft subset.
    pub fn bind_authored_coupled_corrective_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        owners: RelationalAuthoredCoupledOwners,
        correspondence: (
            CommitId,
            (EntityId, CreatedEntityRef),
            Vec<(EntityId, CreatedEntityRef)>,
        ),
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into()).bind_authored_coupled_corrective_root(
            execution,
            installed,
            owners,
            correspondence,
        )
    }

    /// Admit synchronous writers for an ordinary authored source. Unlike a
    /// retirement, an edit does not require every current owner to republish:
    /// Query may prove an output current without appending any commands.
    pub fn readmit_edit_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        source: EntityId,
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_edit_coupled_owners(execution, installed, basis, source)
    }

    /// Bind the admitted edit source to an already-issued member closure.
    /// Retirement/restoration evidence and mandatory linked writers stay exact;
    /// new operand routes use the edit's ordinary current-owner admission.
    pub fn readmit_member_edit_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        members: RelationalAuthoredCoupledOwners,
        source: EntityId,
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_member_edit_coupled_owners(execution, installed, members, source)
    }

    /// Admit synchronous writers for an edit envelope. `prefix` is the authored
    /// prefix's source; `later` are the sources of stages 2..k, which Native
    /// then accepts once each, in this order. Each source is admitted exactly
    /// as a single edit is; the owner set is their union, with a route current
    /// for two sources admitted once.
    pub fn readmit_edit_coupled_stage_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        prefix: EntityId,
        later: &[EntityId],
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_edit_coupled_stage_owners(execution, installed, basis, prefix, later)
    }

    /// Admit the complete later-source roster without replacing the prefix's
    /// actual Edit/Retirement/Restoration custody. Every source uses the same C.
    pub fn readmit_later_authored_stage_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        owners: RelationalAuthoredCoupledOwners,
        later: &[EntityId],
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        AuthoredPublicationReads::new(self.into())
            .readmit_later_authored_stage_owners(execution, installed, owners, later)
    }
}
