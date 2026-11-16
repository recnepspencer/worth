//! The coupled owner lends only installed public Sources on its actual prefix.
use super::*;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::{
    ManagedIndexAppliedSourceRead, ManagedIndexSourcePreparationDenial as Denial,
    ManagedIndexSourceProgram,
};
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Borrow a genuine retained prefix of this same live assembly. Discovery
    /// may still own an earlier prefix; it must not read the later image instead.
    pub fn read_managed_source_at<'candidate>(
        &'candidate self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<ManagedIndexAppliedSourceRead<'candidate>, Denial> {
        let current = self.read.borrow();
        let Some(current) = current.as_ref() else {
            return Err(Denial::ForeignSource);
        };
        if !current
            .candidate_identity()
            .includes_assembly_prefix(pin.candidate_identity())
        {
            return Err(Denial::ForeignSource);
        }
        let mut installed = false;
        for group in self
            .runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .all_source_groups()
        {
            for actual in group.public_programs() {
                execution.check_live().map_err(|cause| {
                    crate::derived_relation::DerivedRelationReadDenial::Request(cause.into())
                })?;
                execution
                    .run_native_with_checkpoint(|context| checkpoint(context, 1))
                    .map_err(|cause| {
                        crate::derived_relation::DerivedRelationReadDenial::Request(cause.into())
                    })?
                    .0
                    .map_err(|cause| {
                        crate::derived_relation::DerivedRelationReadDenial::Request(cause.into())
                    })?;
                installed |= actual == program;
            }
        }
        if !installed {
            return Err(Denial::ForeignSource);
        }
        ManagedIndexAppliedSourceRead::issue(execution, self.identity, pin.clone(), program, scope)
    }
}
