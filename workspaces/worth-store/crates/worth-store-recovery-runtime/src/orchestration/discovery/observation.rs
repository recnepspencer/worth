use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{PhysicalRecordFormatDeclaration, BOOTSTRAP_CATALOG_BYTES};
use worth_store_recovery_physics::{
    PhysicalRecoveryResidue, PhysicalRootSelectorDenial, PhysicalRootSlotObservation,
};

use crate::entry::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits, PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::{
    admit_observed_bootstrap_catalog, IntegrityAdmittedRecoveryArtifact,
    RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::super::manifest_facts::{observe_manifest_facts, ManifestObservationBudget};
use super::super::reader_limit::{OversizedArtifact, ReadCeiling};
use super::super::ManifestFactsDiscovery;
use super::{
    discovery_limit, refused_read, BootstrapDiscovery, CheckpointDiscovery, DiscoveryFailure,
    WalDiscovery,
};

mod checkpoint;
mod counters;
mod root_observation;
mod wal_observation;
#[cfg(all(test, feature = "certification-test-authority"))]
mod wal_pressure_tests;

use wal_observation::observe_wal;

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
    coordination: &mut super::super::RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    record_format: PhysicalRecordFormatDeclaration,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    ingress_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<ObservedSources, DiscoveryFailure> {
    let declaration = limits.declaration();
    let mut roots = {
        let mut allocation = coordination
            .owner_mut()
            .begin_source_read_allocation()
            .map_err(root_observation::window_admission_failure)?;
        observe_root_slots(discovery, limits, record_format, counters, &mut allocation)?
    };
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
    let checkpoint = {
        let mut allocation = coordination
            .owner_mut()
            .begin_source_read_allocation()
            .map_err(checkpoint::window_admission_failure)
            .map_err(&preserve_manifest_observations)?;
        observe_checkpoint(
            discovery,
            limits,
            record_format,
            &mut roots.remaining_manifest_bytes,
            counters,
            ingress_trace,
            &mut allocation,
        )
    }
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
    let ceiling = ReadCeiling::of_artifact(BOOTSTRAP_CATALOG_BYTES as u64);
    let artifact = match discovery.read_bootstrap_catalog(ceiling.requested()) {
        Ok(artifact) => artifact,
        Err(failure) => {
            let OversizedArtifact = refused_read(
                failure,
                ceiling,
                PhysicalRecoveryLimitDimension::ObservationBytes,
            )?;
            // A catalog is its fixed frame and nothing more.
            return Ok(BootstrapDiscovery::Rejected(
                RecoveryIntegrityIngressRejection::NonCanonicalEncoding,
            ));
        }
    };
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

fn observe_root_manifest_facts(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    limits: PhysicalRecoveryLimits,
    roots: &mut RootObservations,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> Result<(ManifestFactsDiscovery, ManifestFactsDiscovery), DiscoveryFailure> {
    let declaration = limits.declaration();
    let mut remaining_manifest_entries = declaration.manifest_entries;
    let mut manifest_blocks = 0;
    let current_manifest_facts_result = observe_manifest_facts(
        discovery,
        &roots.current,
        ManifestObservationBudget {
            remaining_bytes: &mut roots.remaining_manifest_bytes,
            admitted_bytes: declaration.manifest_bytes,
            remaining_entries: &mut remaining_manifest_entries,
            admitted_entries: declaration.manifest_entries,
            blocks_read: &mut manifest_blocks,
        },
    );
    counters.manifest_bytes = declaration.manifest_bytes - roots.remaining_manifest_bytes;
    counters.manifest_entries = declaration.manifest_entries - remaining_manifest_entries;
    counters.manifest_blocks = manifest_blocks;
    let current_manifest_facts = current_manifest_facts_result?;
    let previous_manifest_facts_result = observe_manifest_facts(
        discovery,
        &roots.previous,
        ManifestObservationBudget {
            remaining_bytes: &mut roots.remaining_manifest_bytes,
            admitted_bytes: declaration.manifest_bytes,
            remaining_entries: &mut remaining_manifest_entries,
            admitted_entries: declaration.manifest_entries,
            blocks_read: &mut manifest_blocks,
        },
    );
    counters.manifest_bytes = declaration.manifest_bytes - roots.remaining_manifest_bytes;
    counters.manifest_entries = declaration.manifest_entries - remaining_manifest_entries;
    counters.manifest_blocks = manifest_blocks;
    let previous_manifest_facts = match previous_manifest_facts_result {
        Ok(facts) => facts,
        Err(failure) => {
            let (_, trace) = current_manifest_facts.into_parts();
            return Err(failure.with_integrity_trace(trace));
        }
    };
    Ok((current_manifest_facts, previous_manifest_facts))
}
