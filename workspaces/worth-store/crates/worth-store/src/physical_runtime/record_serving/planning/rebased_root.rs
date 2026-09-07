use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, RecordArtifactFile,
};

use super::super::{
    planning::prepared_payload::PreparedRecordPayloadPlan,
    publication::{PhysicalManifestCapacityTransition, PublicationPlan},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    RecordAppendDenial, RecordAppendError,
};

mod assembly;
mod projection;

pub(in crate::physical_runtime::record_serving) struct RootRebaseContext<'plan> {
    pub(in crate::physical_runtime::record_serving) allocation:
        &'plan worth_store_buffer_pool::OperationAllocationGrant,
    pub(in crate::physical_runtime::record_serving) media: &'plan QualifiedFilesystemMedia,
    pub(in crate::physical_runtime::record_serving) residency:
        super::super::residency::PhysicalResidencyWorkPort,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current_root:
        &'plan DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) current_free_space:
        &'plan DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime::record_serving) capacity_transition:
        PhysicalManifestCapacityTransition,
}

pub(in crate::physical_runtime::record_serving) fn project_settled_root(
    prepared: PreparedRecordPayloadPlan,
    context: RootRebaseContext<'_>,
    candidate: RecordArtifactFile,
) -> Result<
    (PublicationPlan, DurableFreeSpaceManifestHeader),
    (PreparedRecordPayloadPlan, RecordAppendError),
> {
    if prepared.source_root != *context.current_root {
        return Err((
            prepared,
            RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged),
        ));
    }
    let generation = match successor_generation(context.current_root) {
        Ok(generation) => generation,
        Err(cause) => return Err((prepared, cause)),
    };
    if let Err(cause) = require_capacity(&context) {
        return Err((prepared, cause));
    }
    let projected = match projection::project_successor_root(&context, &prepared, generation) {
        Ok(projected) => projected,
        Err(cause) => return Err((prepared, cause)),
    };
    let payload_manifests = prepared.payload_manifests;
    let publication = PublicationPlan {
        generation,
        manifests: Vec::new(),
        root: RecordArtifactFile::RootManifest { generation },
        candidate,
        manifest: context.current_root.clone(),
        root_bytes: Vec::new(),
        previous_selector_candidate: RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Previous,
            publication: candidate_publication(candidate),
        },
        previous_selector_bytes: Vec::new(),
        current_selector_candidate: RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Current,
            publication: candidate_publication(candidate),
        },
        current_selector_bytes: Vec::new(),
        catalog_bytes: Vec::new(),
        observation: prepared.observation,
    };
    let (mut publication, free_space) =
        assembly::assemble_rebased_publication(publication, context, generation, projected);
    publication.manifests.splice(0..0, payload_manifests);
    Ok((publication, free_space))
}

fn candidate_publication(candidate: RecordArtifactFile) -> u64 {
    let RecordArtifactFile::CatalogCandidate { publication } = candidate else {
        unreachable!("root rebase owns one catalog candidate")
    };
    publication
}

fn successor_generation(current: &DurablePhysicalRootManifest) -> Result<u64, RecordAppendError> {
    current
        .generation()
        .checked_add(1)
        .ok_or(RecordAppendError::Denied(
            RecordAppendDenial::RootGenerationExhausted,
        ))
}

fn require_capacity(context: &RootRebaseContext<'_>) -> Result<(), RecordAppendError> {
    if context.capacity_transition == PhysicalManifestCapacityTransition::PreserveCurrent
        && context.placement.manifest_capacity().get() != context.current_root.node_capacity()
    {
        Err(RecordAppendError::Denied(
            RecordAppendDenial::ManifestCapacityMigrationRequired,
        ))
    } else {
        Ok(())
    }
}

pub(super) fn damaged() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}
