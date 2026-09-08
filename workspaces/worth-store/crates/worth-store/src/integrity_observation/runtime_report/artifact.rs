//! Canonical descriptive artifact addresses in the version-one protocol.
use crate::physical_runtime::{
    PhysicalIntegrityScrubTarget, PhysicalIntegrityScrubWindowObservation,
};
use serde_json::{json, Value};
use worth_store_physical_format::{
    PhysicalArtifactReadTarget as Target, RecordArtifactFile as Record,
};
use worth_store_physical_integrity::PhysicalArtifactScope;

pub(super) fn project(
    target: PhysicalIntegrityScrubTarget,
    observation: &PhysicalIntegrityScrubWindowObservation,
) -> Value {
    let scope = target.scope();
    let (path, identity, generation) = address(target, observation);
    json!({"path":path, "family":super::vocabulary::family(scope.artifact_family()),
        "identity":identity, "generation":generation, "range":{"offset":scope.byte_range().offset(),"length":scope.byte_range().length()},
        "duplicates":[], "outcome":super::outcome::project(observation.outcome)})
}

fn address(
    target: PhysicalIntegrityScrubTarget,
    observation: &PhysicalIntegrityScrubWindowObservation,
) -> (String, String, Option<u64>) {
    let scope = target.scope();
    match target.range().target() {
        Target::Record(record) => record_address(record, scope, observation),
        Target::Wal(identity) => {
            let file = worth_store_wal::WalSegmentArtifactIdentity::new(
                worth_store_wal::WalSegmentId::new(identity.segment().get()).unwrap(),
                worth_store_wal::WalSegmentGeneration::new(identity.generation().get()).unwrap(),
            )
            .file_name();
            (
                format!("families/wal/{file}"),
                format!(
                    "wal:{}:{}:{}",
                    identity.segment().get(),
                    identity.generation().get(),
                    scope.byte_range().offset()
                ),
                Some(identity.generation().get()),
            )
        }
        Target::Checkpoint(identity) => (
            "families/checkpoint.current".into(),
            format!(
                "checkpoint:{}:{}:{}",
                identity.sequence(),
                checkpoint_kind(scope),
                scope.byte_range().offset()
            ),
            Some(identity.sequence().get()),
        ),
        Target::PhysicalWork(identity) => {
            let (runtime, generation, operation) = (
                identity.runtime().get(),
                identity.generation().get(),
                identity.operation().get(),
            );
            (format!("families/physical-work/effect-{runtime:016x}-{generation:016x}-{operation:016x}.pending"),
                format!("operation:{runtime:016x}:{generation:016x}:{operation:016x}"), Some(generation))
        }
    }
}

fn record_address(
    record: Record,
    scope: PhysicalArtifactScope,
    observation: &PhysicalIntegrityScrubWindowObservation,
) -> (String, String, Option<u64>) {
    let (directory, identity, generation) = match record {
        Record::BootstrapCatalog => ("families/records", "bootstrap-catalog".into(), None),
        Record::CurrentRootSelector | Record::PreviousRootSelector => {
            let label = if record == Record::CurrentRootSelector {
                "current-selector"
            } else {
                "previous-selector"
            };
            (
                "families/records",
                observation.selector_identity.map_or_else(
                    || label.into(),
                    |identity| format!("selector:{:016x}", identity.get()),
                ),
                None,
            )
        }
        Record::RootManifest { generation } => (
            "families/records/roots",
            format!("root:{generation:016x}"),
            Some(generation),
        ),
        Record::RootRoutingBlock { generation, block } => (
            "families/records/roots",
            format!("block:{block:016x}"),
            Some(generation),
        ),
        Record::SegmentMembershipBlock { generation, block } => (
            "families/records/segment-manifests",
            format!("block:{block:016x}"),
            Some(generation),
        ),
        Record::FreeSpaceMembershipBlock { generation, block } => (
            "families/records/free-space",
            format!("block:{block:016x}"),
            Some(generation),
        ),
        Record::FreeSpaceManifest { generation } => (
            "families/records/free-space",
            format!("free-space:{generation:016x}"),
            Some(generation),
        ),
        Record::Segment { segment, .. } => {
            let page = scope.page_identity().expect("admitted page target");
            (
                "families/records/segments",
                format!("page:{segment:016x}:{:016x}", page.page_id().get()),
                Some(page.generation().get()),
            )
        }
        Record::ExtentManifest { extent, generation } => (
            "families/records/extent-manifests",
            format!("extent:{extent:016x}"),
            Some(generation),
        ),
        Record::Extent { extent, generation } => (
            "families/records/extents",
            format!(
                "extent:{extent:016x}:chunk:{}",
                scope
                    .extent_chunk_coordinate()
                    .expect("admitted chunk target")
                    .ordinal()
            ),
            Some(generation),
        ),
        Record::CatalogCandidate { .. }
        | Record::RootSelectorCandidate { .. }
        | Record::SegmentManifest { .. } => {
            unreachable!("unsupported target shapes cannot be admitted")
        }
    };
    (
        format!("{directory}/{}", record.file_name()),
        identity,
        generation,
    )
}

fn checkpoint_kind(scope: PhysicalArtifactScope) -> u8 {
    use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::*;
    match scope.artifact_family() {
        CheckpointStreamHeader => 1,
        CheckpointDirtyBasis => 2,
        CheckpointBindingCompaction => 3,
        CheckpointBinding => 4,
        CheckpointFooter => 5,
        _ => unreachable!("checkpoint target scope"),
    }
}
