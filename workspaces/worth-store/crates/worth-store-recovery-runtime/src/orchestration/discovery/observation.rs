use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{PhysicalRecordFormatDeclaration, BOOTSTRAP_CATALOG_BYTES};
use worth_store_recovery_physics::{
    PhysicalRecoveryResidue, PhysicalRootSelectorDenial, PhysicalRootSlotObservation,
};

use crate::entry::{
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryLimits, PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::{
    admit_observed_bootstrap_catalog, IntegrityAdmittedRecoveryArtifact,
    RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::super::manifest_facts::{observe_manifest_facts, ManifestObservationBudget};
use super::super::ManifestFactsDiscovery;
use super::wal::{discover_wal_inventory, WalDiscoveryInventoryDenialKind};
use super::{
    discovery_limit, map_discovery_failure, BootstrapDiscovery, CheckpointDiscovery,
    DiscoveryFailure, WalDiscovery,
};

mod checkpoint;
mod counters;
mod root_observation;

use checkpoint::observe_checkpoint;
use counters::{record_checkpoint_counters, record_wal_counters};
use root_observation::{observe_root_slots, RootObservations};

pub(super) struct ObservedSources {
    pub(super) current: PhysicalRootSlotObservation,
    pub(super) previous: PhysicalRootSlotObservation,
    pub(super) bootstrap: BootstrapDiscovery,
    pub(super) current_manifest_facts: ManifestFactsDiscovery,
    pub(super) previous_manifest_facts: ManifestFactsDiscovery,
    pub(super) checkpoint: CheckpointDiscovery,
    pub(super) wal: WalDiscovery,
    pub(super) residue: Vec<PhysicalRecoveryResidue>,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
}

pub(super) fn observe_all(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &super::super::RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    record_format: PhysicalRecordFormatDeclaration,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    ingress_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<ObservedSources, DiscoveryFailure> {
    let declaration = limits.declaration();
    let mut roots = observe_root_slots(discovery, limits, record_format, counters)?;
    let root_protocol_denials = roots.denials.clone();
    let preserve_root_denials =
        |failure: DiscoveryFailure| failure.with_root_protocol_denials(&root_protocol_denials);
    let bootstrap =
        observe_fallback_anchor(discovery, &roots, record_format, counters, ingress_trace)
            .map_err(&preserve_root_denials)?;
    let (current_manifest_facts, previous_manifest_facts) =
        observe_root_manifest_facts(discovery, limits, &mut roots, counters)
            .map_err(&preserve_root_denials)?;
    let preserve_manifest_observations = |failure| {
        preserve_post_manifest_failure(
            failure,
            &root_protocol_denials,
            &current_manifest_facts,
            &previous_manifest_facts,
        )
    };
    let checkpoint = observe_checkpoint(
        discovery,
        limits,
        &mut roots.remaining_manifest_bytes,
        counters,
        ingress_trace,
    )
    .map_err(&preserve_manifest_observations)?;
    counters.manifest_bytes = declaration.manifest_bytes - roots.remaining_manifest_bytes;
    record_checkpoint_counters(counters, &checkpoint);
    let (wal, residue, wal_entries) = observe_wal(discovery, coordination, limits, counters)
        .map_err(&preserve_manifest_observations)?;
    record_wal_counters(counters, &wal, &residue, wal_entries);
    if discovery.counters().bytes_read > declaration.observation_bytes {
        return Err(preserve_manifest_observations(
            discovery_limit(
                PhysicalRecoveryLimitDimension::ObservationBytes,
                discovery.counters().bytes_read,
                declaration.observation_bytes,
            )
            .with_integrity_observations(wal.integrity_observations()),
        ));
    }
    Ok(ObservedSources {
        current: roots.current,
        previous: roots.previous,
        bootstrap,
        current_manifest_facts,
        previous_manifest_facts,
        checkpoint,
        wal,
        residue,
        root_protocol_denials,
    })
}

fn preserve_post_manifest_failure(
    failure: DiscoveryFailure,
    root_protocol_denials: &[PhysicalRecoverySourceDenial],
    current: &ManifestFactsDiscovery,
    previous: &ManifestFactsDiscovery,
) -> DiscoveryFailure {
    failure
        .with_root_protocol_denials(root_protocol_denials)
        .with_integrity_trace(current.integrity_trace().clone())
        .with_integrity_trace(previous.integrity_trace().clone())
}

fn observe_fallback_anchor(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    roots: &RootObservations,
    record_format: PhysicalRecordFormatDeclaration,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    ingress_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<BootstrapDiscovery, DiscoveryFailure> {
    if !current_requires_fallback_anchor(&roots.current)
        || !matches!(roots.previous, PhysicalRootSlotObservation::Candidate(_))
    {
        return Ok(BootstrapDiscovery::NotRequired);
    }
    let artifact = discovery
        .read_bootstrap_catalog(BOOTSTRAP_CATALOG_BYTES as u64)
        .map_err(map_selector_discovery_failure)?;
    let mut ingress = crate::integrity_ingress::RecoveryIntegrityIngressTrace::new();
    let attempt = admit_observed_bootstrap_catalog(
        &artifact,
        discovery.store_identity(),
        record_format,
        ingress.counters_mut(),
    );
    ingress.retain(attempt.observation());
    let discovery = match attempt.into_outcome() {
        Ok(IntegrityAdmittedRecoveryArtifact::BootstrapCatalog(admitted)) => {
            let projection = admitted.project(ingress.counters_mut());
            BootstrapDiscovery::Admitted(
                worth_store_recovery_physics::PhysicalBootstrapFallbackAnchor::from_integrity_projection(
                    admitted.scope().store_identity(),
                    projection.record_format,
                    projection.current_root_generation,
                ),
            )
        }
        Ok(_) => unreachable!("bootstrap ingress routes only the bootstrap family"),
        Err(RecoveryIntegrityIngressRejection::Absent) => BootstrapDiscovery::Absent,
        Err(rejection) => BootstrapDiscovery::Rejected(rejection),
    };
    record_bootstrap_counters(counters, ingress.counters());
    ingress_trace.append(ingress);
    Ok(discovery)
}

fn record_bootstrap_counters(
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    ingress: RecoveryIntegrityIngressCounters,
) {
    counters.bootstrap_integrity_attempts = ingress.attempted;
    counters.bootstrap_integrity_admissions = ingress.admitted;
    counters.bootstrap_absent = ingress.rejected_absent;
    counters.bootstrap_integrity_rejections =
        ingress.attempted - ingress.admitted - ingress.rejected_absent;
    counters.bootstrap_owner_projections = ingress.owner_projection_entries;
    counters.bootstrap_owner_decoder_entries = ingress.owner_decoder_entries;
}

fn current_requires_fallback_anchor(current: &PhysicalRootSlotObservation) -> bool {
    matches!(
        current,
        PhysicalRootSlotObservation::SelectorRejected(
            PhysicalRootSelectorDenial::Integrity | PhysicalRootSelectorDenial::AuthorityMismatch
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_conflict_never_requests_fallback_anchor_io() {
        assert!(!current_requires_fallback_anchor(
            &PhysicalRootSlotObservation::SelectorRejected(PhysicalRootSelectorDenial::Conflict)
        ));
        assert!(current_requires_fallback_anchor(
            &PhysicalRootSlotObservation::SelectorRejected(
                PhysicalRootSelectorDenial::AuthorityMismatch
            )
        ));
    }
}

fn map_selector_discovery_failure(
    failure: worth_store::physical_runtime::RecoveryDiscoveryFailure,
) -> DiscoveryFailure {
    map_discovery_failure(
        failure,
        PhysicalRecoveryLimitDimension::ObservationBytes,
        PhysicalRecoveryLimitDimension::ObservationBytes,
    )
}

fn observe_root_manifest_facts(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    limits: PhysicalRecoveryLimits,
    roots: &mut RootObservations,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<(ManifestFactsDiscovery, ManifestFactsDiscovery), DiscoveryFailure> {
    let declaration = limits.declaration();
    let mut remaining_manifest_entries = declaration.manifest_entries;
    let admitted_manifest_blocks = declaration
        .manifest_entries
        .checked_mul(2)
        .and_then(|value| value.checked_add(2))
        .ok_or(PhysicalRecoveryBlock::DiscoveryLimit)?;
    let mut remaining_manifest_blocks = admitted_manifest_blocks;
    let current_manifest_facts_result = observe_manifest_facts(
        discovery,
        &roots.current,
        ManifestObservationBudget {
            remaining_bytes: &mut roots.remaining_manifest_bytes,
            admitted_bytes: declaration.manifest_bytes,
            remaining_entries: &mut remaining_manifest_entries,
            admitted_entries: declaration.manifest_entries,
            remaining_blocks: &mut remaining_manifest_blocks,
            admitted_blocks: admitted_manifest_blocks,
        },
    );
    counters.manifest_bytes = declaration.manifest_bytes - roots.remaining_manifest_bytes;
    counters.manifest_entries = declaration.manifest_entries - remaining_manifest_entries;
    let current_manifest_facts = current_manifest_facts_result?;
    let previous_manifest_facts_result = observe_manifest_facts(
        discovery,
        &roots.previous,
        ManifestObservationBudget {
            remaining_bytes: &mut roots.remaining_manifest_bytes,
            admitted_bytes: declaration.manifest_bytes,
            remaining_entries: &mut remaining_manifest_entries,
            admitted_entries: declaration.manifest_entries,
            remaining_blocks: &mut remaining_manifest_blocks,
            admitted_blocks: admitted_manifest_blocks,
        },
    );
    counters.manifest_bytes = declaration.manifest_bytes - roots.remaining_manifest_bytes;
    counters.manifest_entries = declaration.manifest_entries - remaining_manifest_entries;
    let previous_manifest_facts = match previous_manifest_facts_result {
        Ok(facts) => facts,
        Err(failure) => {
            let (_, trace) = current_manifest_facts.into_parts();
            return Err(failure.with_integrity_trace(trace));
        }
    };
    counters.manifest_blocks =
        current_manifest_facts.block_count() + previous_manifest_facts.block_count();
    Ok((current_manifest_facts, previous_manifest_facts))
}

fn observe_wal(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &super::super::RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<(WalDiscovery, Vec<PhysicalRecoveryResidue>, u64), DiscoveryFailure> {
    let declaration = limits.declaration();
    let observed = discovery
        .read_wal_artifacts(declaration.wal_segments, declaration.wal_bytes)
        .map_err(|failure| {
            map_discovery_failure(
                failure,
                PhysicalRecoveryLimitDimension::WalSegments,
                PhysicalRecoveryLimitDimension::WalBytes,
            )
        })?;
    let wal_entries = observed.len() as u64;
    let inspected = match discover_wal_inventory(
        coordination.owner(),
        observed,
        discovery.store_identity(),
        declaration.wal_frames,
    ) {
        Ok(inspected) => inspected,
        Err(denial) => {
            let (wal, residue) = finish_wal_inventory(denial.inventory);
            record_wal_counters(counters, &wal, &residue, wal_entries);
            let observations = wal.integrity_observations;
            let failure = match denial.kind {
                WalDiscoveryInventoryDenialKind::CounterOverflow => {
                    DiscoveryFailure::from(PhysicalRecoveryBlock::DiscoveryLimit)
                }
                WalDiscoveryInventoryDenialKind::FrameLimitExceeded { observed, admitted } => {
                    discovery_limit(
                        PhysicalRecoveryLimitDimension::WalFrames,
                        observed,
                        admitted,
                    )
                }
                WalDiscoveryInventoryDenialKind::SourceBinding => {
                    DiscoveryFailure::from(PhysicalRecoveryBlock::WalInventory)
                }
            };
            return Err(failure.with_integrity_observations(observations));
        }
    };
    if inspected.observed_bytes > declaration.wal_bytes {
        let observed_bytes = inspected.observed_bytes;
        let (wal, residue) = finish_wal_inventory(inspected);
        record_wal_counters(counters, &wal, &residue, wal_entries);
        return Err(discovery_limit(
            PhysicalRecoveryLimitDimension::WalBytes,
            observed_bytes,
            declaration.wal_bytes,
        )
        .with_integrity_observations(wal.integrity_observations));
    }
    debug_assert!(inspected.frames_scanned <= declaration.wal_frames);
    let (wal, residue) = finish_wal_inventory(inspected);
    Ok((wal, residue, wal_entries))
}

fn finish_wal_inventory(
    inspected: super::wal::WalDiscoveryInventory,
) -> (WalDiscovery, Vec<PhysicalRecoveryResidue>) {
    let residue = inspected.residue;
    let valid_segments = inspected.candidates.len() as u64;
    let wal = WalDiscovery {
        rejected: !inspected.corruptions.is_empty(),
        candidates: inspected.candidates,
        admitted: inspected.admitted,
        integrity_observations: inspected.observations,
        integrity_ingress: inspected.ingress,
        scanned_frames: inspected.frames_scanned,
        valid_frames: inspected.valid_frames,
        valid_bytes: inspected.valid_bytes,
        observed_bytes: inspected.observed_bytes,
        torn_suffix_frames: inspected.torn_suffix_frames,
        torn_suffix_bytes: inspected.torn_suffix_bytes,
        corruption_denials: inspected.corruptions.len() as u64,
        scanned_segments: inspected.canonical_segments,
        valid_segments,
        corruptions: inspected.corruptions,
    };
    (wal, residue)
}
