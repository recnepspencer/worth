//! Correction DATA admission joins the actual Native and protected World owners.
pub mod native_records;
pub mod original_join;
pub mod relationships;
use super::{canonical_record, WorthQueryCanonicalCorrectionFact};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryAftermathCausalityLayout;
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
    CurrentNativeHead(
        &'a CompositeCommitIdentity,
        &'a std::collections::BTreeMap<
            CommitId,
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
        >,
        &'a std::collections::BTreeSet<CommitId>,
    ),
    SelectedWorldBranch(&'a ProductBranchObservation),
}
impl JoinScope<'_> {
    fn selected(&self) -> &CompositeCommitIdentity {
        match self {
            Self::CurrentNativeHead(selected, _, _) => selected,
            Self::SelectedWorldBranch(branch) => branch.selected_commit(),
        }
    }
}
pub fn readmit_latest(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    originals: &WorldProgramActivationHistory,
    selected: &CompositeCommitIdentity,
    material: Option<&super::super::WorthQueryPrimaryGraphProvider>,
    metadata_authority: Option<&worth_relational::facade::durability::HomeReadmissionMetadataAuthority>,
) -> Result<Option<RecordRef>, Denial> {
    let empty_map = std::collections::BTreeMap::new();
    let empty_set = std::collections::BTreeSet::new();
    readmit_latest_with_unpublished(
        execution,
        native,
        layout,
        originals,
        selected,
        material,
        (&empty_map, &empty_set),
        metadata_authority,
    )
}
pub fn readmit_latest_with_unpublished(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    originals: &WorldProgramActivationHistory,
    selected: &CompositeCommitIdentity,
    material: Option<&super::super::WorthQueryPrimaryGraphProvider>,
    unpublished: (
        &std::collections::BTreeMap<
            CommitId,
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
        >,
        &std::collections::BTreeSet<CommitId>,
    ),
    metadata_authority: Option<&worth_relational::facade::durability::HomeReadmissionMetadataAuthority>,
) -> Result<Option<RecordRef>, Denial> {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .map_err(|cause| source_denial(format!("correction current Native source: {cause:?}")))?;
    readmit(
        execution,
        native,
        layout,
        originals,
        JoinScope::CurrentNativeHead(selected, unpublished.0, unpublished.1),
        &basis,
        material,
        metadata_authority,
    )
}
pub fn readmit_branch(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    originals: &WorldProgramActivationHistory,
    branch: &ProductBranchObservation,
    material: Option<&super::super::WorthQueryPrimaryGraphProvider>,
    metadata_authority: Option<&worth_relational::facade::durability::HomeReadmissionMetadataAuthority>,
) -> Result<Option<RecordRef>, Denial> {
    readmit(
        execution,
        native,
        layout,
        originals,
        JoinScope::SelectedWorldBranch(branch),
        branch.basis().relational_basis(),
        material,
        metadata_authority,
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
    metadata_authority: Option<&worth_relational::facade::durability::HomeReadmissionMetadataAuthority>,
) -> Result<Option<RecordRef>, Denial> {
    let owner = scope
        .selected()
        .owner_identity()
        .durable()
        .ok_or(Denial::ForeignWorldInventory)?;
    let mut facts = native_records::read(
        execution,
        native,
        layout,
        basis,
        WorldOwnerCoordinate::Home { owner },
        metadata_authority,
    )?;
    if let JoinScope::CurrentNativeHead(_, continuable, all_unpublished) = scope {
        // Released-only unpublished Native rows cannot continue; continuable rows
        // remain in `facts` so full coordinate, relationship, and capture checks run.
        facts.retain(|fact| {
            !all_unpublished.contains(&fact.child.commit_id)
                || continuable.contains_key(&fact.child.commit_id)
        });
    }
    let facts = original_join::readmit(execution, originals, facts, &scope)?;
    relationships::readmit(&facts)?;
    if let Some(provider) = material {
        for fact in facts.values() {
            match provider.admit_correction_lineage_material(&fact.child) {
                Ok(()) => continue,
                Err(crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialDenial::Unavailable) => {
                    let continuable_attempt = match scope {
                        JoinScope::CurrentNativeHead(_, continuable, _) => {
                            continuable.get(&fact.child.commit_id).copied()
                        }
                        JoinScope::SelectedWorldBranch(_) => None,
                    };
                    let attempt = continuable_attempt.or_else(|| {
                        originals
                            .original_observations()
                            .find(|obs| obs.relational_commit() == Some(fact.child.commit_id))
                            .and_then(|obs| {
                                originals
                                    .original_states()
                                    .find(|state| state.reserved_commit() == obs.identity())
                            })
                            .map(|state| state.association().attempt())
                    });
                    if !attempt.is_some_and(|a| provider.has_captured_correction_material(a)) {
                        return Err(Denial::RetainedMaterialRequired {
                            record: fact.record.clone(),
                        });
                    }
                }
                Err(_) => {
                    return Err(Denial::RetainedMaterialRequired {
                        record: fact.record.clone(),
                    });
                }
            }
        }
    }
    Ok(facts.values().next().map(|fact| fact.record.clone()))
}
fn source_denial(subject: impl Into<String>) -> Denial {
    Denial::Source(crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial::new(
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected, subject))
}
