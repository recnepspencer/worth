//! Mutation-target affinity checks have no application or entity type parameter.
use super::super::native_read::NativeIdentity;
use super::super::{
    WorthQueryInvariantMutationTarget, WorthQueryInvariantProjectionTraversalDenial,
};
use crate::domain_computation::{
    authorization::WorthQueryOperationAdmissionIdentity,
    execution_runtime::WorthQueryRuntimeAuthorityIdentity,
};
use std::{marker::PhantomData, sync::Arc};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
pub(super) fn target(
    identity: NativeIdentity<'_>,
    authority: u64,
    phase: Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: &ApplicationSchemaBindingIdentity,
    admission: Option<WorthQueryOperationAdmissionIdentity>,
) -> Result<WorthQueryInvariantMutationTarget<(), ()>, WorthQueryInvariantProjectionTraversalDenial>
{
    let admission_identity = admission.ok_or_else(|| {
        WorthQueryInvariantProjectionTraversalDenial::mutation_target_unavailable(
            identity.entity.as_ref(),
        )
    })?;
    if identity.authority != authority {
        return Err(
            WorthQueryInvariantProjectionTraversalDenial::foreign_identity(
                identity.entity.as_ref(),
            ),
        );
    }
    Ok(WorthQueryInvariantMutationTarget {
        entity_id: identity.entity_id,
        phase,
        entity: Arc::clone(identity.entity),
        runtime_authority: runtime,
        binding_identity: binding.clone(),
        admission_identity,
        _marker: PhantomData,
    })
}
impl WorthQueryInvariantMutationTarget<(), ()> {
    #[inline(always)]
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn into_typed<S, E>(
        self,
    ) -> WorthQueryInvariantMutationTarget<S, E> {
        WorthQueryInvariantMutationTarget {
            entity_id: self.entity_id,
            phase: self.phase,
            entity: self.entity,
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity,
            admission_identity: self.admission_identity,
            _marker: PhantomData,
        }
    }
}
