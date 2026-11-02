//! Exact applied-record comparison admits reuse, never committed Source READ.
use super::{ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram};
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::derived_relation::DerivedRelationReadDenial as ReadDenial;
use crate::runtime::{PositionedRelationalSnapshot, RelationalPreparationRuntime};
use crate::validation::engine::state_view::InvariantStateView;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_proof::{
    AdmittedManagedIndexSourceCorrespondence, ManagedIndexSourceCorrespondenceAuthority,
    ManagedIndexSourceCorrespondenceIssuer,
};

mod records;

/// The proof and its exact selected receiver stay in this one preparation call.
pub(super) struct NativeManagedSourceCorrespondence<'a> {
    owner: &'a super::NativeManagedSourceInterpretation,
    selected: PositionedRelationalSnapshot,
    generation: u64,
    authority: ManagedIndexSourceCorrespondenceAuthority,
    proof: AdmittedManagedIndexSourceCorrespondence,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(super) fn prepare<'a>(
    draft: &'a AppliedMutationDraft,
    execution: ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    after: &InvariantStateView<'_>,
    generation: u64,
) -> Result<Option<NativeManagedSourceCorrespondence<'a>>, Denial> {
    let owner = &draft.source_interpretation;
    if !owner.has_restoration_origin() {
        return Ok(None);
    }
    let source = owner
        .materialization_source()
        .ok_or(Denial::ForeignSource)?;
    let basis = owner.physical_basis();
    if !std::sync::Arc::ptr_eq(
        &basis.inner.root.retained_schema_authority(),
        &source.basis().inner.root.retained_schema_authority(),
    ) {
        return Ok(None);
    }
    let (equal, _) = execution
        .run_native_with_checkpoint(|context| {
            records::matches(draft, execution, runtime, source, after, context)
        })
        .map_err(|cause| ReadDenial::Request(cause.into()))?;
    if !equal? {
        return Ok(None);
    }
    let identity = basis.identity();
    let branch_bytes = identity.branch_id().0.len();
    let bytes = branch_bytes
        .checked_add(2 * size_of::<usize>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)?;
    let memory = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| ReadDenial::Request(cause.into()))?;
    let (position, _) = execution
        .run_native_with_checkpoint(|context| {
            charge(context, bytes.checked_add(7).ok_or_else(overflow)?)?;
            Ok::<_, Denial>(
                basis
                    .observation()
                    .commit_id()
                    .and_then(|commit| runtime.history.canonical_stream_position(commit)),
            )
        })
        .map_err(|cause| ReadDenial::Request(cause.into()))?;
    let Some(position) = position? else {
        return Ok(None);
    };
    let selected = PositionedRelationalSnapshot {
        runtime_instance_id: identity.runtime_instance_id(),
        branch_id: identity.branch_id().clone(),
        root_id: basis.inner.root.id(),
        version_id: basis.observation().version_id(),
        commit_id: basis.observation().commit_id(),
        position: Some(position),
    };
    let issuer = ManagedIndexSourceCorrespondenceIssuer::fresh();
    Ok(Some(NativeManagedSourceCorrespondence {
        owner,
        selected,
        generation,
        authority: issuer.authority(),
        proof: issuer.admit(),
        _memory: memory,
    }))
}

impl NativeManagedSourceCorrespondence<'_> {
    pub(super) fn root(
        &self,
        program: &ManagedIndexSourceProgram,
        scope: crate::identity::data::EntityId,
    ) -> Result<Option<super::preparation::RootPayload>, Denial> {
        if !self.authority.admits(&self.proof) {
            return Err(Denial::ForeignSource);
        }
        let Some(image) = self
            .owner
            .physical_basis()
            .inner
            .root
            .managed_source_image
            .get()
        else {
            return Ok(None);
        };
        image.corresponding_payload(&self.selected, self.generation, program, scope)
    }
}

fn overflow() -> Denial {
    ReadDenial::Definition(crate::derived_relation::DerivedRelationDenial::WorkBudgetExceeded)
        .into()
}
fn charge(context: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    context
        .checkpoint(work)
        .map_err(|cause| ReadDenial::Request(cause.into()).into())
}
