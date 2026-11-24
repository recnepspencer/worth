//! Current Native custody for an authored anchor and its complete derived closure.
use super::*;
use contracts::identity::data::record_ids::EntityId;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug)]
pub struct RelationalAuthoredRetirement {
    pub(crate) roles: Vec<RequiredOutputRoleTarget>,
    pub(crate) anchor: EntityId,
    pub(crate) owned_members: bool,
    pub(crate) birth: contracts::history::data::coordinates::CommitId,
    pub(crate) co_anchors: Vec<(EntityId, contracts::history::data::coordinates::CommitId)>,
    pub(crate) before: AdmittedRelationalBranchBasis,
    pub(crate) publications: Vec<RelationalRequiredOutputPublication>,
}

impl RelationalAuthoredRetirement {
    pub fn authorities(
        &self,
    ) -> impl Iterator<Item = &worth_proof::RequiredOutputPublicationAuthority> {
        self.publications
            .iter()
            .map(|publication| publication.authority())
    }
}

impl AuthoredPublicationReads<'_> {
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
        self.readmit_authored_source_retirement_with_co_anchors(
            execution,
            installed,
            basis,
            anchor,
            &[],
            roles,
        )
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
        self.readmit_authored_retirement(
            execution,
            installed,
            basis,
            (anchor, co_anchors),
            roles,
            &[],
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
        kinds: &[super::AuthoredOwnedMemberKind],
    ) -> Result<Option<RelationalAuthoredRetirement>, TransactionCommitError> {
        self.validate_authored_owned_members(execution, basis, anchors.0, kinds, roles)?;
        self.readmit_authored_retirement(execution, installed, basis, anchors, roles, kinds)
    }

    fn readmit_authored_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        anchors: (EntityId, &[EntityId]),
        roles: &[RequiredOutputRoleTarget],
        kinds: &[super::AuthoredOwnedMemberKind],
    ) -> Result<Option<RelationalAuthoredRetirement>, TransactionCommitError> {
        let (anchor, co_anchors) = anchors;
        let owned_members = !kinds.is_empty();
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial("foreign installed source-retirement owner"));
        }
        if installed.source_scope_ownership.is_empty() && !owned_members {
            return Ok(None);
        }
        if !installed.source_scope_ownership.is_empty() {
            installed.require_current_ownership_storage(basis, anchor)?;
        }
        // Selection and every cold predecessor readmission share one original request scope.
        execution
            .in_scope(|_| {
                let crate::publication::ownership::authored_retirement::SelectedAuthoredRetirement {
                    birth,
                    co_anchors,
                    retirements: selected,
                    ..
                } = crate::publication::ownership::authored_retirement::select_authored_source_retirement(
                    execution,
                    self.history,
                    installed,
                    basis,
                    anchor,
                    co_anchors,
                    roles,
                    crate::publication::ownership::authored_retirement::AuthoredRetirementSelection::SourceScopeClosure,
                )?;
                if selected.is_empty() && !owned_members {
                    return Ok(None);
                }
                let publications = selected
                    .iter()
                    .map(|envelope| {
                        self.readmit_canonical_publication(
                            execution,
                            installed,
                            basis,
                            &envelope.origin,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Some(RelationalAuthoredRetirement {
                    roles: roles.to_vec(),
                    anchor,
                    owned_members,
                    birth,
                    co_anchors,
                    before: basis.clone(),
                    publications,
                }))
            })
            .map_err(custody::execution_stop)?
    }
}

#[doc(hidden)]
pub trait RelationalAuthoredRetirementFieldsPort: Sized {
    fn publications(&self) -> &Vec<RelationalRequiredOutputPublication>;
}

impl RelationalAuthoredRetirementFieldsPort for RelationalAuthoredRetirement {
    fn publications(&self) -> &Vec<RelationalRequiredOutputPublication> {
        &self.publications
    }
}

#[doc(hidden)]
pub trait RelationalAuthoredRetirementFixturePort {
    fn replace_selected_origin_for_test(
        &mut self,
        index: usize,
        origin: std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
    );
}

impl RelationalAuthoredRetirementFixturePort for RelationalAuthoredRetirement {
    /// Alter only the selected member's origin for the closed restoration court.
    fn replace_selected_origin_for_test(
        &mut self,
        index: usize,
        origin: std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
    ) {
        self.publications[index].origin = origin;
    }
}
