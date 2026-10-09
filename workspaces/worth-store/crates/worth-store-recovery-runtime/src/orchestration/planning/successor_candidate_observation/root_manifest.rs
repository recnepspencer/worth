use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::artifact_read::read as read_artifact;
use super::denial::invalid;
use super::materialization::CandidateMaterialization;
use super::resident::memory_failure;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};
use crate::orchestration::planning::manifest_entry_budget::{pays_for, ChargeToken};
use crate::progression::PlanningResidentAllowance;

pub(super) struct ObservedSuccessorRoot {
    pub(super) artifact: RecordArtifactFile,
    pub(super) manifest: DurablePhysicalRootManifest,
    pub(super) bytes: Vec<u8>,
}

/// The generation the selected root's successor carries. The last
/// generation has none: its root is then the artifact that is invalid.
pub(super) fn successor_generation(
    selected: &DurablePhysicalRootManifest,
) -> Result<u64, PhysicalRecoverySuccessorCandidateDenial> {
    selected.generation().checked_add(1).ok_or_else(|| {
        invalid(RecordArtifactFile::RootManifest {
            generation: selected.generation(),
        })
    })
}

/// Probes the root of `generation`, paid by `charge` before it is read. An
/// absent root is its own last read: the caller spends the entry, since
/// looking is what it paid for.
pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    charge: &ChargeToken,
    materialization: &mut CandidateMaterialization,
    root_protocol_counters: &mut crate::entry::PhysicalRecoveryRootProtocolCounters,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Option<ObservedSuccessorRoot>, PhysicalRecoverySuccessorCandidateDenial> {
    pays_for(charge, generation);
    let artifact = RecordArtifactFile::RootManifest { generation };
    let source = read_artifact(discovery, artifact, format, charge, allowance)?;
    if source.bytes().is_none() {
        return Ok(None);
    }
    // Root admission decodes in place, then canonically re-encodes in bind.
    // Both Format-owned encoding buffers coexist with the retained source,
    // including when a decode-valid source fails the canonical comparison.
    allowance
        .transient(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64)
        .map_err(|failure| memory_failure(artifact, failure))?;
    let admitted = admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&source),
        discovery.store_identity(),
        format,
        generation,
    )
    .map_err(
        |rejection| PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
            artifact,
            generation,
            denial: rejection.diagnostic(),
        },
    )?;
    root_protocol_counters.record_successor_root_integrity_admission();
    let (manifest, _) = admitted.project();
    root_protocol_counters.record_successor_root_interpretation();
    let bytes = source
        .into_bytes()
        .expect("source-bound root admission retained a present observation");
    materialization.retain_root(bytes.len());
    materialization.retain_reference();
    Ok(Some(ObservedSuccessorRoot {
        artifact,
        manifest,
        bytes,
    }))
}
