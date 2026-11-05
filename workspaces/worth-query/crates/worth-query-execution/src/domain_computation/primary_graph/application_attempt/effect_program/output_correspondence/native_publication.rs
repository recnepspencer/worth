//! Bind descriptive Query roles to the selected publication's actual Native roles.
use super::*;
use worth_relational::facade::mvcc::{
    CompanionPreflightStop as Stop, RelationalRequiredOutputPublication,
    RequiredOutputRolePosture as NativePosture,
};
impl WorthQueryApplicationOutputCorrespondence {
    pub(in crate::domain_computation::primary_graph) fn matches_native_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        publication: &RelationalRequiredOutputPublication,
    ) -> Result<bool, Stop> {
        if self.storage.roles.len() != publication.members().len() {
            return Ok(false);
        }
        execution
            .run_native_with_checkpoint(|context| {
                let depth = self
                    .storage
                    .roles
                    .len()
                    .saturating_add(1)
                    .checked_ilog2()
                    .unwrap_or(0) as u64
                    + 1;
                for (role, name, posture, entity) in publication.members() {
                    let work = (role.len() as u64)
                        .saturating_add(1)
                        .saturating_mul(depth)
                        .saturating_add(name.len() as u64 + 1);
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context, work,
                    )
                    .map_err(|stop| {
                        Stop::Execution(worth_execution::WorkCeilingDenial::Stopped(stop).into())
                    })?;
                    let Some(binding) = self.storage.binding(role) else {
                        return Ok(false);
                    };
                    let expected = match posture {
                        NativePosture::Create => WorthQueryApplicationOutputPosture::Create,
                        NativePosture::Preserve => WorthQueryApplicationOutputPosture::Preserve,
                        NativePosture::Retire => WorthQueryApplicationOutputPosture::Retire,
                    };
                    // Native selection is a fresh Preserve of the original
                    // live role; Query keeps its original performed posture.
                    let posture_matches = binding.posture == expected
                        || (publication.is_historical_selection()
                            && expected == WorthQueryApplicationOutputPosture::Preserve
                            && binding.posture == WorthQueryApplicationOutputPosture::Create);
                    if binding.entity != entity || binding.entity_name != name || !posture_matches {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .map_err(|cause| Stop::Execution(cause.into()))?
            .0
    }
}
