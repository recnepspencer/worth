//! Correction DATA admission joins the actual Native and protected World owners.
mod home;
mod native_records;
mod original_join;
mod relationships;
use super::{canonical_record, WorthQueryCanonicalCorrectionFact};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryAftermathCausalityLayout;
pub(in crate::domain_computation::primary_graph) use home::{verify, verify_source};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis, history::CommitId, identity::EntityId,
    runtime::RelationalRuntime, snapshots::SnapshotHandle, transactions::RecordRef,
};
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchObservation, WorldOriginalObservation,
    WorldOriginalObservationDenial, WorldProgramActivationHistory,
};
#[derive(Debug)]
pub enum QueryHomeCorrectionLineageDenial {
    Source(crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial),
    Population(worth_relational::facade::runtime::EntityKindTruthReadLimitExceeded),
    Snapshot(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
    SnapshotRelease {
        snapshot: SnapshotHandle,
        cause: worth_relational::facade::snapshots::RelationalSnapshotReleaseDenial,
        preceding: Option<Box<Self>>,
    },
    Record {
        entity: EntityId,
        kind: worth_relational::facade::identity::KindId,
        cause: super::WorthQueryAftermathCausalityReadDenial,
    },
    MissingWorldOriginal {
        record: RecordRef,
    },
    AmbiguousWorldOriginal {
        record: RecordRef,
    },
    WorldOriginalMismatch {
        record: RecordRef,
    },
    OutsideOriginalAncestry {
        record: RecordRef,
    },
    World(WorldOriginalObservationDenial),
    WorldSource(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::WorldReadmissionDenial,
        >,
    ),
    Inventory(
        worth_runtime_world::facade::RuntimeWorldServiceDenial<
            worth_runtime_world::facade::ProductBranchInventoryDenial,
        >,
    ),
    ForeignWorldInventory,
    InconsistentRelationships {
        record: RecordRef,
    },
    /// Structural lineage has joined; the retained material owner must still admit
    /// its own input, pre-image and original-meaning obligations before service.
    RetainedMaterialRequired {
        record: RecordRef,
    },
}
use QueryHomeCorrectionLineageDenial as Denial;
enum JoinScope<'a> {
    CurrentNativeHead(&'a CompositeCommitIdentity),
    SelectedWorldBranch(&'a ProductBranchObservation),
}
impl JoinScope<'_> {
    fn selected(&self) -> &CompositeCommitIdentity {
        match self {
            Self::CurrentNativeHead(selected) => selected,
            Self::SelectedWorldBranch(branch) => branch.selected_commit(),
        }
    }
}
fn readmit_latest(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    originals: &WorldProgramActivationHistory,
    selected: &CompositeCommitIdentity,
    material: Option<&super::super::WorthQueryPrimaryGraphProvider>,
) -> Result<Option<RecordRef>, Denial> {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .map_err(|cause| source_denial(format!("correction current Native source: {cause:?}")))?;
    readmit(
        execution,
        native,
        layout,
        originals,
        JoinScope::CurrentNativeHead(selected),
        &basis,
        material,
    )
}
fn readmit_branch(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    originals: &WorldProgramActivationHistory,
    branch: &ProductBranchObservation,
    material: Option<&super::super::WorthQueryPrimaryGraphProvider>,
) -> Result<Option<RecordRef>, Denial> {
    readmit(
        execution,
        native,
        layout,
        originals,
        JoinScope::SelectedWorldBranch(branch),
        branch.basis().relational_basis(),
        material,
    )
}
fn readmit(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    originals: &WorldProgramActivationHistory,
    scope: JoinScope<'_>,
    basis: &AdmittedRelationalBranchBasis,
    material: Option<&super::super::WorthQueryPrimaryGraphProvider>,
) -> Result<Option<RecordRef>, Denial> {
    let owner = scope
        .selected()
        .owner_identity()
        .durable()
        .ok_or(Denial::ForeignWorldInventory)?;
    let facts = native_records::read(
        execution,
        native,
        layout,
        basis,
        WorldOwnerCoordinate::Home { owner },
    )?;
    let facts = original_join::readmit(execution, originals, facts, &scope)?;
    relationships::readmit(&facts)?;
    if let Some(provider) = material {
        for fact in facts.values() {
            provider
                .admit_correction_lineage_material(&fact.child)
                .map_err(|_| Denial::RetainedMaterialRequired {
                    record: fact.record.clone(),
                })?;
        }
    }
    Ok(facts.values().next().map(|fact| fact.record.clone()))
}
fn source_denial(subject: impl Into<String>) -> Denial {
    Denial::Source(crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial::new(
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected, subject))
}
