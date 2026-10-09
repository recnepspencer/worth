use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration,
    DurableFreeSpaceManifestHeader, DurableRootSelector, RecordArtifactFile, RootSelectorIdentity,
    RootSelectorRole,
};

use super::{projection::ProjectedSuccessorRoot, RootRebaseContext};
use crate::physical_runtime::record_serving::publication::PublicationPlan;

pub(super) struct RootAssemblyContext<'a> {
    pub(super) media: &'a worth_store_physical_backend::QualifiedFilesystemMedia,
    pub(super) format: crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat,
    pub(super) current_root: &'a worth_store_physical_format::DurablePhysicalRootManifest,
}

pub(super) fn assemble_rebased_publication(
    publication: PublicationPlan,
    context: RootRebaseContext<'_>,
    generation: u64,
    projected: ProjectedSuccessorRoot,
) -> (PublicationPlan, DurableFreeSpaceManifestHeader) {
    assemble_successor(
        publication,
        RootAssemblyContext {
            media: context.media,
            format: context.format,
            current_root: context.current_root,
        },
        generation,
        projected,
    )
}

pub(super) fn assemble_successor(
    mut publication: PublicationPlan,
    context: RootAssemblyContext<'_>,
    generation: u64,
    projected: ProjectedSuccessorRoot,
) -> (PublicationPlan, DurableFreeSpaceManifestHeader) {
    projected.observe_discovery(&mut publication.observation);
    publication.generation = generation;
    publication.root = RecordArtifactFile::RootManifest { generation };
    publication.manifest = projected.root;
    publication.root_bytes = publication.manifest.encode(context.format.declaration());
    publication.catalog_bytes = BootstrapCatalog::new(
        context.media.store_identity(),
        context.format.declaration(),
        CurrentRootCatalogEntry::new(
            CurrentRootCatalogGeneration::new(generation).expect("successor is nonzero"),
        ),
    )
    .encode()
    .to_vec();
    let previous_generation = context.current_root.generation();
    let previous_identity = RootSelectorIdentity::new(previous_generation)
        .expect("published root generation is nonzero");
    let current_identity =
        RootSelectorIdentity::new(generation).expect("successor root generation is nonzero");
    publication.previous_selector_bytes = DurableRootSelector::new(
        context.media.store_identity(),
        context.format.declaration(),
        previous_identity,
        RootSelectorRole::Previous,
        previous_generation,
        Some(current_identity),
        Some(generation),
    )
    .expect("successor-linked previous selector is valid")
    .encode()
    .to_vec();
    publication.current_selector_bytes = DurableRootSelector::new(
        context.media.store_identity(),
        context.format.declaration(),
        current_identity,
        RootSelectorRole::Current,
        generation,
        Some(previous_identity),
        Some(previous_generation),
    )
    .expect("previous-linked current selector is valid")
    .encode()
    .to_vec();
    publication.manifests = projected
        .manifests
        .into_iter()
        .map(|(artifact, bytes)| {
            let coordinate = worth_store_physical_format::RecordFrameCoordinate::new(
                artifact,
                0,
                u32::try_from(bytes.len()).expect("bounded manifest length"),
            )
            .expect("manifest coordinate");
            (coordinate, bytes)
        })
        .collect();
    publication.routing_metadata_bytes = publication.retained_metadata_bytes();
    (publication, projected.free_space)
}
