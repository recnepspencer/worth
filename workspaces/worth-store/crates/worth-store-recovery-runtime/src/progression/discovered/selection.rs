use worth_store_recovery_physics::{
    admit_physical_page_facts, admit_physical_wal_tail, select_physical_recovery_sources,
    PhysicalCheckpointBase, PhysicalRecoveryResidue, PhysicalRootSlotObservation,
    PhysicalSourceSelection, SelectedCompactionProduct, SelectedPhysicalPageFacts,
    SelectedPhysicalRoot, SelectedPhysicalRootRole, SelectedPhysicalWalTail,
};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryLimits, PhysicalRecoverySourceDenial,
};
use crate::orchestration::{
    AdmittedWalInventory, BootstrapDiscovery, CheckpointDiscovery, ManifestFactsDiscovery,
    ManifestFactsState, WalDiscovery,
};
use worth_store::physical_runtime::StoreRecoveryCheckpointBindingBasis;

use super::PhysicalRecoveryDiscoveryCounters;

mod checkpoint;
mod failure;
mod root;

use checkpoint::select_checkpoint;
use failure::SelectionFailure;

pub(super) struct SelectionInput {
    pub(super) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    pub(super) current: PhysicalRootSlotObservation,
    pub(super) previous: PhysicalRootSlotObservation,
    pub(super) bootstrap: BootstrapDiscovery,
    pub(super) current_manifest_facts: ManifestFactsDiscovery,
    pub(super) previous_manifest_facts: ManifestFactsDiscovery,
    pub(super) checkpoint: CheckpointDiscovery,
    pub(super) wal: WalDiscovery,
    pub(super) residue: Vec<PhysicalRecoveryResidue>,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(super) counters: PhysicalRecoveryDiscoveryCounters,
}

pub(super) struct SelectionOutput {
    pub(super) selection: PhysicalSourceSelection,
    pub(super) admitted_wal: AdmittedWalInventory,
    pub(super) wal_integrity_observations:
        Vec<crate::entry::PhysicalRecoveryWalIntegrityObservation>,
    pub(super) checkpoint_binding_basis: Option<StoreRecoveryCheckpointBindingBasis>,
    pub(super) counters: PhysicalRecoveryDiscoveryCounters,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(super) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

struct ManifestSelectionInput {
    current: ManifestFactsState,
    previous: ManifestFactsState,
    limits: PhysicalRecoveryLimits,
}

struct FinalSelectionInput {
    root: SelectedPhysicalRoot,
    page_facts: SelectedPhysicalPageFacts,
    retained_previous_page_facts: Option<SelectedPhysicalPageFacts>,
    checkpoint: Option<PhysicalCheckpointBase>,
    wal_tail: SelectedPhysicalWalTail,
    compaction: Option<SelectedCompactionProduct>,
    residue: Vec<PhysicalRecoveryResidue>,
    counters: PhysicalRecoveryDiscoveryCounters,
    frontier: u64,
}
pub(super) fn select_sources(
    input: SelectionInput,
    limits: PhysicalRecoveryLimits,
) -> Result<SelectionOutput, SelectionFailure> {
    let mut counters = input.counters;
    let mut integrity_trace = input.integrity_trace;
    let (current_manifest_facts, current_integrity_trace) =
        input.current_manifest_facts.into_parts();
    integrity_trace.append(current_integrity_trace);
    let (previous_manifest_facts, previous_integrity_trace) =
        input.previous_manifest_facts.into_parts();
    integrity_trace.append(previous_integrity_trace);
    let wal_integrity_observations = input.wal.integrity_observations();
    let (root, root_protocol_denials) = root::select(
        input.current,
        input.previous,
        input.bootstrap,
        input.root_protocol_denials,
        counters,
    )
    .map_err(|failure| {
        failure
            .with_integrity_trace(integrity_trace.clone())
            .with_integrity_observations(wal_integrity_observations.clone())
    })?;
    let (page_facts, retained_previous_page_facts) = select_manifest_facts(
        &root,
        ManifestSelectionInput {
            current: current_manifest_facts,
            previous: previous_manifest_facts,
            limits,
        },
        &mut counters,
    )
    .map_err(|failure| {
        failure
            .with_root_protocol_denials(&root_protocol_denials)
            .with_integrity_trace(integrity_trace.clone())
            .with_integrity_observations(wal_integrity_observations.clone())
    })?;
    let (checkpoint, checkpoint_binding_basis) =
        select_checkpoint(&root, input.checkpoint, counters, &mut integrity_trace).map_err(
            |failure| {
                failure
                    .with_root_protocol_denials(&root_protocol_denials)
                    .with_integrity_trace(integrity_trace.clone())
                    .with_integrity_observations(wal_integrity_observations.clone())
            },
        )?;
    let frontier = checkpoint
        .as_ref()
        .map_or(0, |checkpoint| checkpoint.wal_tail_begin_lsn());
    let (wal_tail, admitted_wal, wal_integrity_observations) =
        select_wal(&root, input.wal, frontier, &mut counters).map_err(|failure| {
            failure
                .with_root_protocol_denials(&root_protocol_denials)
                .with_integrity_trace(integrity_trace.clone())
        })?;
    let compaction = checkpoint.as_ref().map(SelectedCompactionProduct::admit);
    let selection = select_final_cut(FinalSelectionInput {
        root,
        page_facts,
        retained_previous_page_facts,
        checkpoint,
        wal_tail,
        compaction,
        residue: input.residue,
        counters,
        frontier,
    })
    .map_err(|failure| {
        failure
            .with_root_protocol_denials(&root_protocol_denials)
            .with_integrity_trace(integrity_trace.clone())
            .with_integrity_observations(wal_integrity_observations.clone())
    })?;
    Ok(SelectionOutput {
        selection,
        admitted_wal,
        wal_integrity_observations,
        checkpoint_binding_basis,
        counters,
        root_protocol_denials,
        integrity_trace,
    })
}

fn select_manifest_facts(
    root: &SelectedPhysicalRoot,
    input: ManifestSelectionInput,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<(SelectedPhysicalPageFacts, Option<SelectedPhysicalPageFacts>), SelectionFailure> {
    let (selected, retained_previous) = match root.role() {
        SelectedPhysicalRootRole::Current => (input.current, Some(input.previous)),
        SelectedPhysicalRootRole::PreviousFallback => (input.previous, None),
    };
    let generation = root.selected().selector().root_generation();
    let blocks = match selected {
        ManifestFactsState::Observed { blocks } => blocks,
        ManifestFactsState::Rejected(denial) => {
            return Err(SelectionFailure::new(
                PhysicalRecoveryBlockKind::SourceSelection,
                *counters,
                "records/root routing blocks",
            )
            .with_generation(generation)
            .with_source_denials(vec![
                PhysicalRecoverySourceDenial::ManifestObservation(denial),
            ]));
        }
        ManifestFactsState::Unavailable => {
            return Err(SelectionFailure::new(
                PhysicalRecoveryBlockKind::SourceSelection,
                *counters,
                "records/root routing blocks",
            )
            .with_generation(generation));
        }
    };
    let declaration = input.limits.declaration();
    let page_facts = admit_physical_page_facts(
        root.selected(),
        blocks,
        declaration.manifest_entries,
        declaration.distinct_pages_and_extents,
    )
    .map_err(|denial| manifest_failure(denial, generation, *counters, declaration))?;
    counters.selected_page_facts = page_facts.placements().len() as u64;
    counters.distinct_pages_and_extents = page_facts.distinct_pages_and_extents();
    let retained_previous_page_facts = match (root.retained_previous(), retained_previous) {
        (Some(previous), Some(ManifestFactsState::Observed { blocks })) => Some(
            admit_physical_page_facts(
                previous,
                blocks,
                declaration.manifest_entries,
                declaration.distinct_pages_and_extents,
            )
            .map_err(|denial| {
                manifest_failure(
                    denial,
                    previous.selector().root_generation(),
                    *counters,
                    declaration,
                )
            })?,
        ),
        (Some(previous), Some(ManifestFactsState::Rejected(denial))) => {
            return Err(SelectionFailure::new(
                PhysicalRecoveryBlockKind::SourceSelection,
                *counters,
                "records/retained-previous routing blocks",
            )
            .with_generation(previous.selector().root_generation())
            .with_source_denials(vec![
                PhysicalRecoverySourceDenial::ManifestObservation(denial),
            ]));
        }
        (Some(previous), Some(ManifestFactsState::Unavailable)) => {
            return Err(SelectionFailure::new(
                PhysicalRecoveryBlockKind::SourceSelection,
                *counters,
                "records/retained-previous routing blocks",
            )
            .with_generation(previous.selector().root_generation()));
        }
        (None, _) | (_, None) => None,
    };
    Ok((page_facts, retained_previous_page_facts))
}

fn select_wal(
    root: &SelectedPhysicalRoot,
    wal: WalDiscovery,
    frontier: u64,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<
    (
        SelectedPhysicalWalTail,
        AdmittedWalInventory,
        Vec<crate::entry::PhysicalRecoveryWalIntegrityObservation>,
    ),
    SelectionFailure,
> {
    let generation = root.selected().selector().root_generation();
    let (candidates, rejected, corruptions, admitted, observations) = wal.into_selection_parts();
    if rejected {
        let denials = corruptions
            .into_iter()
            .map(PhysicalRecoverySourceDenial::WalIntegrity)
            .collect();
        return Err(SelectionFailure::new(
            PhysicalRecoveryBlockKind::WalInventory,
            *counters,
            "families/wal",
        )
        .with_generation(generation)
        .with_lsn(frontier)
        .with_integrity_observations(observations)
        .with_source_denials(denials));
    }
    match admit_physical_wal_tail(frontier, candidates) {
        Ok(selected) => Ok((selected, admitted, observations)),
        Err(denial) => {
            counters.wal_missing_range_denials += 1;
            Err(SelectionFailure::new(
                PhysicalRecoveryBlockKind::WalInventory,
                *counters,
                "families/wal",
            )
            .with_generation(generation)
            .with_lsn(frontier)
            .with_integrity_observations(observations)
            .with_source_denials(vec![PhysicalRecoverySourceDenial::WalTail(denial)]))
        }
    }
}

fn select_final_cut(
    input: FinalSelectionInput,
) -> Result<PhysicalSourceSelection, SelectionFailure> {
    let generation = input.root.selected().selector().root_generation();
    select_physical_recovery_sources(
        input.root,
        input.page_facts,
        input.retained_previous_page_facts,
        input.checkpoint,
        input.wal_tail,
        input.compaction,
        input.residue,
    )
    .map_err(|denial| {
        SelectionFailure::new(
            PhysicalRecoveryBlockKind::SourceSelection,
            input.counters,
            "selected persisted-source cut",
        )
        .with_generation(generation)
        .with_lsn(input.frontier)
        .with_source_denials(vec![PhysicalRecoverySourceDenial::FinalSelection(denial)])
    })
}

fn manifest_failure(
    denial: worth_store_recovery_physics::PhysicalPageFactDenial,
    generation: u64,
    counters: PhysicalRecoveryDiscoveryCounters,
    limits: crate::entry::PhysicalRecoveryLimitDeclaration,
) -> SelectionFailure {
    let mut failure = SelectionFailure::new(
        PhysicalRecoveryBlockKind::SourceSelection,
        counters,
        "records/root routing blocks",
    )
    .with_generation(generation)
    .with_source_denials(vec![PhysicalRecoverySourceDenial::ManifestFacts(denial)]);
    match denial {
        worth_store_recovery_physics::PhysicalPageFactDenial::ManifestEntryLimit => {
            failure.kind = PhysicalRecoveryBlockKind::DiscoveryLimit;
            failure.evidence.limit = Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
                observed: limits.manifest_entries + 1,
                admitted: limits.manifest_entries,
            });
        }
        worth_store_recovery_physics::PhysicalPageFactDenial::DistinctPageOrExtentLimit => {
            failure.kind = PhysicalRecoveryBlockKind::DiscoveryLimit;
            failure.evidence.limit = Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::DistinctPagesAndExtents,
                observed: limits.distinct_pages_and_extents + 1,
                admitted: limits.distinct_pages_and_extents,
            });
        }
        _ => {}
    }
    failure
}
