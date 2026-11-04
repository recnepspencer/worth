//! Project original roles only after the protected original fold has verified.
use super::{verified_originals::VerifiedCheckpointOriginals, Denial};
use crate::durability::checkpoint_admission::{AdmissionRowKey, EncodedAdmissionRow};
mod outputs;
pub(in crate::durability) mod reference;
pub(in crate::durability) mod root_directory;
use serde::{ser::SerializeSeq, Serialize, Serializer};
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_foundational::facade::{CheckpointAdmissionCut, CheckpointAdmissionScope};
use worth_proof::NativeCheckpointOriginalsVerified;

/// Native's own verified-original capture. It has no byte or field constructor.
/// Source selection and a receiver still have to admit its emitted artifact.
pub struct CapturedRelationalCheckpointAdmissionImage {
    rows: ExecutionArray<EncodedAdmissionRow>,
    source: crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact,
    normalized: std::sync::Arc<
        crate::durability::source_initial_inputs::normalized::NormalizedOriginalInputs,
    >,
    capture_base: Option<
        std::sync::Arc<crate::durability::checkpoint_admission::PreparedCheckpointCaptureBase>,
    >,
    roots: ExecutionImmutableBytes,
    checkpoint_digest: [u8; 32],
    source_digest: [u8; 32],
    initial_digest: [u8; 32],
    covered: u64,
    transitions: u64,
}
/// The only constructor follows successful emission from this original producer.
/// Descriptive source/image bytes have no conversion into this witness payload.
pub(in crate::durability) struct SealedCheckpointCaptureData {
    publication:
        std::sync::Arc<crate::durability::checkpoint_admission::PreparedCheckpointCaptureBase>,
    source: crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact,
    image: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
    normalized: std::sync::Arc<
        crate::durability::source_initial_inputs::normalized::NormalizedOriginalInputs,
    >,
}
impl worth_proof::AuthorityMarker for SealedCheckpointCaptureData {}
impl SealedCheckpointCaptureData {
    pub(in crate::durability) fn into_parts(
        self,
    ) -> (
        std::sync::Arc<crate::durability::checkpoint_admission::PreparedCheckpointCaptureBase>,
        crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact,
        crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
        std::sync::Arc<
            crate::durability::source_initial_inputs::normalized::NormalizedOriginalInputs,
        >,
    ) {
        (self.publication, self.source, self.image, self.normalized)
    }
}
impl std::fmt::Debug for CapturedRelationalCheckpointAdmissionImage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CapturedRelationalCheckpointAdmissionImage")
            .field("rows", &self.rows.len())
            .field("covered", &self.covered)
            .field("transitions", &self.transitions)
            .finish_non_exhaustive()
    }
}

impl CapturedRelationalCheckpointAdmissionImage {
    pub fn seal_home(
        &self,
        home: worth_foundational::facade::HomeIdentity,
        world_through: [u64; 3],
        world_image_digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
        crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal,
    > {
        self.seal_home_cut(
            &CheckpointAdmissionCut::new(
                CheckpointAdmissionScope::Home {
                    home,
                    native_origin: self.initial_digest,
                    complete_source_digest: self.source_digest,
                    world_through,
                    world_image_digest,
                },
                self.covered,
                self.transitions,
                self.checkpoint_digest,
            ),
            request,
        )
    }
    pub fn seal_home_cut(
        &self,
        cut: &CheckpointAdmissionCut,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::checkpoint_admission::RelationalCheckpointAdmissionArtifact,
        crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal,
    > {
        use crate::durability::checkpoint_admission::{
            CheckpointAdmissionBindingMismatch as Mismatch,
            RelationalCheckpointAdmissionRefusal as Refusal,
        };
        request
            .consult()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let (native_origin, complete_source_digest) = match cut.scope() {
            CheckpointAdmissionScope::Home {
                native_origin,
                complete_source_digest,
                ..
            } => (native_origin, complete_source_digest),
            CheckpointAdmissionScope::Interim { .. } => {
                return Err(Refusal::BindingMismatch(Mismatch::Scope))
            }
        };
        if native_origin != &self.initial_digest {
            return Err(Refusal::BindingMismatch(Mismatch::NativeOrigin));
        }
        if complete_source_digest != &self.source_digest {
            return Err(Refusal::BindingMismatch(Mismatch::Source));
        }
        if cut.canonical_through() != self.covered || cut.transitions_through() != self.transitions
        {
            return Err(Refusal::BindingMismatch(Mismatch::Cut));
        }
        if cut.checkpoint_digest() != &self.checkpoint_digest {
            return Err(Refusal::BindingMismatch(Mismatch::Checkpoint));
        }
        let image =
            crate::durability::checkpoint_admission::emit(cut, &self.roots, &self.rows, request)?;
        if let Some(slot) = &self.capture_base {
            let sealed =
                NativeCheckpointOriginalsVerified::from_owner(SealedCheckpointCaptureData {
                    publication: slot.clone(),
                    source: self.source.clone(),
                    image: image.clone(),
                    normalized: self.normalized.clone(),
                });
            slot.install(sealed, request)?;
        }
        Ok(image)
    }
}

pub(super) fn capture(
    originals: &NativeCheckpointOriginalsVerified<VerifiedCheckpointOriginals>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedRelationalCheckpointAdmissionImage, Denial> {
    let originals = originals.value().custody();
    let covered = originals
        .original
        .base
        .as_ref()
        .map_or(0, |base| base.directory.manifest.commits.len());
    let mut rows =
        ExecutionRequestArrayBuilder::allocate(*request, covered + originals.commits.len())
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    if let Some(base) = &originals.original.base {
        let carried = base
            .state
            .image()
            .carry_rows(request)
            .map_err(admission_stop)?;
        for row in carried {
            rows.push(row)
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
    }
    for original in originals.commits.iter() {
        crate::durability::artifact_payload_emission::request::account_work(request, 1)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        let envelope = &original.envelope;
        let descriptor = original.artifact.roots();
        let visibility = crate::branch::checkpoint_visibility::from_original_commit_data(
            envelope, descriptor, request,
        )
        .map_err(super::root_stop)?;
        let carrier = originals
            .carriers
            .get(
                &(original.inputs.result_schema.as_ref()
                    as *const crate::branch::RelationalBranchRootSchemaAuthority
                    as usize),
            )
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let predecessor = envelope
            .branch_cell_checkpoint
            .as_ref()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let generation = predecessor
            .observation
            .generation()
            .checked_advance()
            .map_err(|_| Denial::OriginalAuthorityUnavailable)?;
        let truth = predecessor
            .truth_version
            .checked_advance()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let created = created_patches(&envelope.patch.authoritative_record_patches, request)?;
        let correctness = match crate::branch::RelationalRootCorrectnessIndex::AuthoritativeFallback
        {
            crate::branch::RelationalRootCorrectnessIndex::AuthoritativeFallback => 0,
        };
        let groups = [
            encode(envelope.semantic_result(), request)?,
            encode(
                &Axes {
                    predecessor: reference::predecessor(predecessor)?,
                    branch: &envelope.branch_context,
                    generation,
                    truth,
                    posture: crate::branch::RelationalBranchBasisPosture::Live,
                },
                request,
            )?,
            encode(&envelope.association_discovery(), request)?,
            encode(
                &Coordinates {
                    receipt: &envelope.commit,
                    position: original.position,
                    digest: original.original_payload_digest,
                    roots: original.artifact.roots(),
                },
                request,
            )?,
            encode(
                &Schema {
                    version: envelope.schema_version,
                    semantics: envelope.descriptor_semantics_version,
                    carrier: *carrier,
                },
                request,
            )?,
            encode(
                &Visibility {
                    storage: *descriptor.truth_root(),
                    schema: *descriptor.schema_root(),
                    correctness,
                    commitment: visibility.digest(),
                },
                request,
            )?,
            encode(&Created(&created), request)?,
            encode(&outputs::Projection(envelope), request)?,
        ];
        rows.push(EncodedAdmissionRow {
            key: AdmissionRowKey {
                position: original.position.0,
                commit: envelope.commit.commit_id.0,
            },
            groups,
        })
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    }
    let roots = root_directory::encode(&originals.manifest, &originals.roots, request)?;
    let normalized = crate::durability::source_initial_inputs::normalized_admission::readmit(
        originals.source_artifact.normalized_bytes(),
        originals.source_artifact.covered().0,
        originals.source_artifact.covered_transitions().0,
        request,
    )
    .map_err(super::prepare::initial_stop)?;
    Ok(CapturedRelationalCheckpointAdmissionImage {
        source: originals.source_artifact.clone(),
        normalized,
        capture_base: originals
            .checkpoint
            .manifest_publication()
            .and_then(|publication| publication.capture_base()),
        rows: rows
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?,
        roots,
        checkpoint_digest: digest(originals.checkpoint.bytes(), request)?,
        source_digest: digest(originals.source_artifact.as_bytes(), request)?,
        initial_digest: originals.source_artifact.initial_coordinate().1,
        covered: originals.source_artifact.covered().0,
        transitions: originals.source_artifact.covered_transitions().0,
    })
}

fn encode(
    value: &impl Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Denial> {
    crate::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        &crate::durability::checkpoint_admission::binary_encoding::BinaryFixedArrays(value),
        request,
    )
    .map_err(crate::durability::checkpoint_history::capture::capture_stop)
}
fn digest(bytes: &[u8], request: &ExecutionRequest<'_, '_>) -> Result<[u8; 32], Denial> {
    crate::durability::checkpoint_admission::capture_digest(bytes, request)
}

#[derive(Serialize)]
struct Axes<'a> {
    predecessor: reference::Predecessor<'a>,
    branch: &'a crate::history::data::BranchId,
    generation: worth_foundational::FoundationalBranchReferenceGeneration,
    truth: crate::branch::RelationalBranchVersion,
    posture: crate::branch::RelationalBranchBasisPosture,
}
#[derive(Serialize)]
struct Coordinates<'a> {
    receipt: &'a crate::history::data::RelationalCommitReceipt,
    position: crate::publication::patch::data::PatchStreamPosition,
    digest: [u8; 32],
    roots: &'a crate::branch::RelationalBranchRootDescriptor,
}
#[derive(Serialize)]
struct Schema {
    version: crate::schema::data::SchemaVersionId,
    semantics: crate::schema::data::DescriptorSemanticsVersion,
    carrier: [u8; 32],
}
#[derive(Serialize)]
struct Visibility {
    storage: [u8; 32],
    schema: [u8; 32],
    correctness: u8,
    commitment: [u8; 32],
}
struct Created<'a>(&'a [&'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch]);
impl Serialize for Created<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for patch in self.0 {
            sequence.serialize_element(patch)?;
        }
        sequence.end()
    }
}

fn created_patches<'a>(
    patches: &'a [crate::publication::patch::data::PublishedAuthoritativeRecordPatch],
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    ExecutionArray<&'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch>,
    Denial,
> {
    use crate::publication::patch::data::RecordStructuralChange::Created;
    let mut count = 0_usize;
    for patch in patches {
        crate::durability::artifact_payload_emission::request::account_work(request, 1)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        if patch.structural_change == Created {
            count += 1;
        }
    }
    let mut selected = ExecutionRequestArrayBuilder::allocate(*request, count)
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    for patch in patches {
        crate::durability::artifact_payload_emission::request::account_work(request, 1)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        if patch.structural_change == Created {
            selected
                .push(patch)
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
    }
    selected
        .seal()
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)
}

fn admission_stop(
    cause: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal,
) -> Denial {
    match cause {
        crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal::Request(
            cause,
        ) => Denial::Request(cause),
        _ => Denial::OriginalAuthorityUnavailable,
    }
}
