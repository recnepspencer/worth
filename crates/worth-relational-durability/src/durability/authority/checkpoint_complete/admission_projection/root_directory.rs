//! The retained root roster carries logical references and exact immutable leaves.
use super::reference;
use crate::durability::authority::checkpoint_complete::roots::CapturedRoots;
use crate::durability::data::RelationalCheckpointManifest;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::CanonicalDigestId;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::checkpoint_region::RelationalCheckpointRegion;

pub(in crate::internal_port::durability::authority::checkpoint_complete) fn encode(
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
                    schema: CanonicalDigestId::new(header.schema_carrier_digest),
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
    commit: contracts::history::data::coordinates::CommitId,
    components: Components<'a>,
    schema: CanonicalDigestId,
    regions: &'a [RelationalCheckpointRegion],
}
struct Components<'a>(&'a [worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor]);
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
                schema: CanonicalDigestId::new(header.schema_carrier_digest),
            })?;
        }
        sequence.end()
    }
}
#[derive(Serialize)]
struct Empty<'a> {
    descriptor: reference::Component<'a>,
    schema: CanonicalDigestId,
}

/// Join authenticated projections to the SAME CHK18 directory. The manifest's
/// serialized canonical leaf is not used to authenticate these meanings.
pub(in crate::internal_port::durability) fn validate_manifest(
    roots: &[u8],
    manifest: &RelationalCheckpointManifest,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    (),
    contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
> {
    use contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
    use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
    let select = |name| {
        contracts::durability::data::recovery_stop::account_work(request, roots.len() as u64)
            .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)?;
        BorrowedPayload(roots)
            .field(name, request)
            .map_err(contracts::durability::checkpoint_admission::refusal::projected_stop)
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
                .map_err(contracts::durability::checkpoint_admission::refusal::projected_stop)?;
            require_projection(&Header {
                commit: header.commit_id, components: Components(&header.recorded_components),
                schema: CanonicalDigestId::new(header.schema_carrier_digest), regions: &header.regions,
            }, first.ok_or(Refusal::InvalidProjectedMeaning)?.0, request)?;
            ordinal += 1;
            Ok(())
        })();
        if let Err(cause) = result {
            refused = Some(cause);
            return Err(contracts::durability::canonical_commit_artifact::borrowed_payload::malformed());
        }
        Ok(())
    }).map_err(|cause| refused.take().unwrap_or(match cause {
        contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal::Request(cause) => Refusal::Request(cause),
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
) -> Result<
    (),
    contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
> {
    let expected = contracts::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        &contracts::durability::checkpoint_admission::binary_encoding::BinaryFixedArrays(value),
        request,
    )
    .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)?;
    if expected.as_ref() != actual {
        return Err(contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::InvalidProjectedMeaning);
    }
    Ok(())
}
