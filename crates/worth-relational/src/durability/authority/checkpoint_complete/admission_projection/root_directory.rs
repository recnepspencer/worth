//! The retained root roster carries logical references and exact immutable leaves.
use super::{reference, Denial};
use crate::durability::{
    authority::checkpoint_complete::roots::FoldedRoots,
    data::{RelationalCheckpointManifest, RelationalCheckpointRegion},
};
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

pub(super) fn encode(
    manifest: &RelationalCheckpointManifest,
    originals: &FoldedRoots,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Denial> {
    super::encode(
        &RootDirectory {
            manifest,
            originals,
        },
        request,
    )
}

struct RootDirectory<'a> {
    manifest: &'a RelationalCheckpointManifest,
    originals: &'a FoldedRoots,
}
impl Serialize for RootDirectory<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_struct("CheckpointRootAdmissionDirectory", 3)?;
        record.serialize_field("roots", &RootRows(self))?;
        record.serialize_field("empty", &EmptyRows(self.manifest))?;
        record.serialize_field("schemas", &self.manifest.schemas)?;
        record.end()
    }
}
struct RootRows<'a>(&'a RootDirectory<'a>);
impl Serialize for RootRows<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.manifest.roots.len()))?;
        for header in &self.0.manifest.roots {
            let original = self
                .0
                .originals
                .get(header.commit_id)
                .ok_or_else(|| serde::ser::Error::custom("verified retained root missing"))?;
            sequence.serialize_element(&(
                Header {
                    commit: header.commit_id,
                    components: Components(&header.recorded_components),
                    schema: header.schema_carrier_digest,
                    regions: &header.regions,
                },
                original.root.output_publication_positions.checkpoint_view(),
            ))?;
        }
        sequence.end()
    }
}
#[derive(Serialize)]
struct Header<'a> {
    commit: crate::history::data::CommitId,
    components: Components<'a>,
    schema: [u8; 32],
    regions: &'a [RelationalCheckpointRegion],
}
struct Components<'a>(&'a [crate::branch::RelationalBranchBasisDescriptor]);
impl Serialize for Components<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for descriptor in self.0 {
            sequence.serialize_element(&reference::component(descriptor))?;
        }
        sequence.end()
    }
}
struct EmptyRows<'a>(&'a RelationalCheckpointManifest);
impl Serialize for EmptyRows<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence =
            serializer.serialize_seq(Some(self.0.retained_empty_components.len()))?;
        for header in &self.0.retained_empty_components {
            sequence.serialize_element(&Empty {
                descriptor: reference::component(&header.descriptor),
                schema: header.schema_carrier_digest,
            })?;
        }
        sequence.end()
    }
}
#[derive(Serialize)]
struct Empty<'a> {
    descriptor: reference::Component<'a>,
    schema: [u8; 32],
}
