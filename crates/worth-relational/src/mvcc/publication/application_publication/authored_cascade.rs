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
        if installed.source_scope_ownership.is_empty() {
            return Ok(None);
        }
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial("foreign installed source-retirement owner"));
        }
        // Selection and every cold predecessor readmission share one original request scope.
        execution
            .in_scope(|_| {
                let (birth, selected) = crate::history::data::select_authored_source_retirement(
                    execution, self, installed, basis, anchor, roles,
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
                    before: basis.clone(),
                    publications,
                }))
            })
            .map_err(custody::execution_stop)?
    }
}
