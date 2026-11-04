//! The retained root roster carries logical references and exact immutable leaves.
use super::{reference, Denial};
use crate::durability::{
    authority::checkpoint_complete::roots::CapturedRoots,
    data::{RelationalCheckpointManifest, RelationalCheckpointRegion},
};
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

pub(super) fn encode(
    manifest: &RelationalCheckpointManifest,
    originals: &CapturedRoots,
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
    originals: &'a CapturedRoots,
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
                original.output_publication_positions.checkpoint_view(),
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

/// Join authenticated projections to the SAME CHK18 directory. The manifest's
/// serialized canonical leaf is not used to authenticate these meanings.
pub(in crate::durability) fn validate_manifest(
    roots: &[u8],
    manifest: &RelationalCheckpointManifest,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal> {
    use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
    use crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal as Refusal;
    let select = |name| {
        crate::durability::artifact_payload_emission::request::account_work(
            request,
            roots.len() as u64,
        )
        .map_err(crate::durability::checkpoint_admission::native_stop)?;
        BorrowedPayload(roots)
            .field(name, request)
            .map_err(crate::durability::checkpoint_admission::projected_stop)
    };
    require_projection(&manifest.schemas, select("schemas")?.0, request)?;
    require_projection(&EmptyRows(manifest), select("empty")?.0, request)?;
    let mut ordinal = 0;
    let mut refused = None;
    select("roots")?.elements(request, |row| {
        let result = (|| {
            let header = manifest.roots.get(ordinal).ok_or(Refusal::InvalidProjectedMeaning)?;
            let mut first = None;
            row.elements(request, |value| { if first.is_none() { first = Some(value); } Ok(()) })
                .map_err(crate::durability::checkpoint_admission::projected_stop)?;
            require_projection(&Header {
                commit: header.commit_id, components: Components(&header.recorded_components),
                schema: header.schema_carrier_digest, regions: &header.regions,
            }, first.ok_or(Refusal::InvalidProjectedMeaning)?.0, request)?;
            ordinal += 1;
            Ok(())
        })();
        if let Err(cause) = result {
            refused = Some(cause);
            return Err(crate::durability::canonical_commit_artifact::borrowed_payload::malformed());
        }
        Ok(())
    }).map_err(|cause| refused.take().unwrap_or_else(|| match cause {
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause) => Refusal::Request(cause),
        _ => Refusal::InvalidProjectedMeaning,
    }))?;
    if ordinal != manifest.roots.len() {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    Ok(())
}

fn require_projection(
    value: &impl Serialize,
    actual: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal> {
    let expected = crate::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        &crate::durability::checkpoint_admission::binary_encoding::BinaryFixedArrays(value),
        request,
    )
    .map_err(crate::durability::checkpoint_admission::native_stop)?;
    if expected.as_ref() != actual {
        return Err(crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal::InvalidProjectedMeaning);
    }
    Ok(())
}
