//! Current Native custody for an authored anchor and its complete derived closure.
use super::*;
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::EntityId, runtime::RelationalRuntime,
};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub struct RelationalAuthoredRetirement {
    pub(crate) roles: Vec<RequiredOutputRoleTarget>,
    pub(crate) anchor: EntityId,
    pub(crate) birth: crate::history::data::CommitId,
    pub(crate) co_anchors: Vec<(EntityId, crate::history::data::CommitId)>,
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
        if installed.source_scope_ownership.is_empty() {
            return Ok(None);
        }
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial("foreign installed source-retirement owner"));
        }
        // Selection and every cold predecessor readmission share one original request scope.
        execution
            .in_scope(|_| {
                let crate::history::data::SelectedAuthoredRetirement {
                    birth,
                    co_anchors,
                    retirements: selected,
                    ..
                } = crate::history::data::select_authored_source_retirement(
                    execution,
                    self,
                    installed,
                    basis,
                    anchor,
                    co_anchors,
                    roles,
                    crate::history::data::AuthoredRetirementSelection::SourceScopeClosure,
                )?;
                if selected.is_empty() {
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
                    birth,
                    co_anchors,
                    before: basis.clone(),
                    publications,
                }))
            })
            .map_err(custody::execution_stop)?
    }
}
