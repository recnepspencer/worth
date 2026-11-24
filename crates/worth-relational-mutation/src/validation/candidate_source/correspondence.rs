//! Exact applied-record comparison admits reuse, never committed Source READ.
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::positioned_snapshot::{PositionedRelationalSnapshot, PositionedRelationalSnapshotParts, assemble_positioned_relational_snapshot};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use super::{ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram};
use crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial as ReadDenial;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::validation::engine::state_view::InvariantStateView;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_proof::{
    AdmittedManagedIndexSourceCorrespondence, ManagedIndexSourceCorrespondenceAuthority,
    ManagedIndexSourceCorrespondenceIssuer,
};

pub mod records;

/// The proof and its exact selected receiver stay in this one preparation call.
pub struct NativeManagedSourceCorrespondence<'a> {
    owner: &'a super::NativeManagedSourceInterpretation,
    selected: PositionedRelationalSnapshot,
    generation: u64,
    authority: ManagedIndexSourceCorrespondenceAuthority,
    proof: AdmittedManagedIndexSourceCorrespondence,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Only the already-applied record scope and selected symbol owner are compared.
pub struct AppliedCorrespondenceInputs<'a> {
    pub owner: &'a super::NativeManagedSourceInterpretation,
    pub changed_records: &'a [RecordRef],
    pub symbols: &'a RuntimeSymbolTable,
}

pub fn prepare<'a>(
    inputs: AppliedCorrespondenceInputs<'a>,
    execution: ExecutionRequest<'_, '_>,
    after: &InvariantStateView<'_>,
    generation: u64,
    canonical_position: impl FnOnce(CommitId) -> Option<PatchStreamPosition>,
) -> Result<Option<NativeManagedSourceCorrespondence<'a>>, Denial> {
    let owner = inputs.owner;
    if !owner.has_restoration_origin() {
        return Ok(None);
    }
    let source = owner
        .materialization_source()
        .ok_or(Denial::ForeignSource)?;
    let basis = owner.physical_basis();
    if !std::sync::Arc::ptr_eq(
        &basis.owner_inner().root.retained_schema_authority(),
        &source
            .basis()
            .owner_inner()
            .root
            .retained_schema_authority(),
    ) {
        return Ok(None);
    }
    let (equal, _) = execution
        .run_native_with_checkpoint(|context| {
            records::matches(
                inputs.changed_records,
                execution,
                inputs.symbols,
                source,
                after,
                context,
            )
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
            Ok::<_, Denial>(basis.observation().commit_id().and_then(canonical_position))
        })
        .map_err(|cause| ReadDenial::Request(cause.into()))?;
    let Some(position) = position? else {
        return Ok(None);
    };
    let selected = assemble_positioned_relational_snapshot(PositionedRelationalSnapshotParts {
        runtime_instance_id: identity.runtime_instance_id(),
        branch_id: identity.branch_id().clone(),
        root_id: basis.owner_inner().root.id(),
        version_id: basis.observation().version_id(),
        commit_id: basis.observation().commit_id(),
        position: Some(position),
    });
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
        scope: crate::contracts::identity::data::record_ids::EntityId,
    ) -> Result<Option<super::preparation::RootPayload>, Denial> {
        if !self.authority.admits(&self.proof) {
            return Err(Denial::ForeignSource);
        }
        let Some(image) = self
            .owner
            .physical_basis()
            .owner_inner()
            .root
            .managed_source_image
            .for_branch(self.selected.branch_id())
        else {
            return Ok(None);
        };
        image.corresponding_payload(&self.selected, self.generation, program, scope)
    }
}

fn overflow() -> Denial {
    ReadDenial::Definition(
        crate::contracts::derived_relation::definition::DerivedRelationDenial::WorkBudgetExceeded,
    )
    .into()
}
fn charge(context: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    context
        .checkpoint(work)
        .map_err(|cause| ReadDenial::Request(cause.into()).into())
}
