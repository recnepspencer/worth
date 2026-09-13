use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::artifact_read::optional_source;
use super::denial::invalid;
use super::materialization::CandidateMaterialization;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};

pub(super) struct ObservedSuccessorRoot {
    pub(super) artifact: RecordArtifactFile,
    pub(super) manifest: DurablePhysicalRootManifest,
    pub(super) bytes: Vec<u8>,
}

pub(super) fn read(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    materialization: &mut CandidateMaterialization,
    root_protocol_counters: &mut crate::entry::PhysicalRecoveryRootProtocolCounters,
) -> Result<Option<ObservedSuccessorRoot>, PhysicalRecoverySuccessorCandidateDenial> {
    let generation = selected.generation().checked_add(1).ok_or_else(|| {
        invalid(RecordArtifactFile::RootManifest {
            generation: selected.generation(),
        })
    })?;
    let artifact = RecordArtifactFile::RootManifest { generation };
    let Some(source) = optional_source(
        discovery.read_root_manifest(generation, byte_limit),
        artifact,
    )?
    else {
        return Ok(None);
    };
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
        .bytes()
        .expect("source-bound root admission retained a present observation")
        .to_vec();
    materialization.retain_root(bytes.len());
    materialization.retain_reference();
    Ok(Some(ObservedSuccessorRoot {
        artifact,
        manifest,
        bytes,
    }))
}
