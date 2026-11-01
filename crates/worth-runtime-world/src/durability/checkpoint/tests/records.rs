//! Untrusted wire descriptions only: no Native basis, physical source or runtime.
use super::*;
use crate::durability::{
    DurableWorldHead, RecordedWorldInitialComponents, WorldOpenAttemptRecord,
    WorldOpenComponentIntent, WorldRecord, WorldRelationalEvidence,
};
use worth_foundational::facade::*;

const COMPONENTS: &[u8] = br#"{
  "relational": {
    "descriptor": {
      "descriptor_version": 2, "runtime_instance_id": 0, "branch_id": "primary",
      "reference": {"branch_id": "relational/0/primary", "target": "Empty", "generation": 0},
      "truth_version": 0, "root_identity": 0,
      "visibility_commitment": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      "posture": "Live"
    }, "target": "Empty"
  },
  "signal": {
    "version": 1, "root": "InitialEmpty", "schema_commitment": "untrusted-schema",
    "visibility_commitment": "signal-initial-empty-visibility-v1",
    "definition_commitment": "signal-initial-empty-definition-v1"
  },
  "bridge": {
    "version": 1, "baseline": "bridge-runtime-world-uninstalled-baseline-v1",
    "adapter_semantic_identity": "untrusted-adapter", "graph_role": "primary"
  }
}"#;

pub(super) fn descriptions(
    request: &ExecutionRequest<'_, '_>,
) -> Result<RecordedWorldInitialComponents, crate::durability::WorldDurableMediumDenial> {
    RecordedWorldInitialComponents::from_json_in(request, COMPONENTS)
}

pub(super) fn origin(
    request: &ExecutionRequest<'_, '_>,
) -> (WorldCheckpointImage, DurableWorldHead) {
    let owner = DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(0),
    );
    let binding = crate::durability::WorldDurableHomeBinding::from_admitted_adapter(
        owner,
        HomeIncarnation::initial(),
        [3; 32],
    );
    let mut image = WorldCheckpointImage::empty(&binding);
    admit(&mut image, &WorldRecord::Owner { owner }, 1, request).unwrap();
    let head = DurableWorldHead {
        branch: DurableProductBranchIdentity::from_recorded(owner, "primary".into()).unwrap(),
        incarnation: DurableProductBranchIncarnation::from_recorded(owner, 1),
        generation: DurableProductReferenceGeneration::from_recorded(0),
        commit: DurableCompositeCommitIdentity::from_recorded(owner, 1),
    };
    let record = WorldRecord::Branch {
        head: head.clone(),
        bootstrap: DurableWorldBootstrapAttemptIdentity::from_recorded(owner, 1),
        program_revision: [3; 32],
        components: descriptions(request).unwrap(),
    };
    admit(&mut image, &record, 2, request).unwrap();
    (image, head)
}
pub(super) fn open(head: &DurableWorldHead, commit: u64) -> WorldOpenAttemptRecord {
    let owner = head.branch.owner();
    WorldOpenAttemptRecord {
        component_intent: WorldOpenComponentIntent::RelationalOnly,
        association: CompositeAttemptAssociation::from_recorded(
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, 1),
            head.branch.clone(),
            head.incarnation,
        )
        .unwrap(),
        commit: DurableCompositeCommitIdentity::from_recorded(owner, commit),
        custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, 1),
        expected: head.clone(),
        program_revision: [3; 32],
    }
}
pub(super) fn published(
    request: &ExecutionRequest<'_, '_>,
) -> (WorldCheckpointImage, WorldOpenAttemptRecord) {
    let (mut image, head) = origin(request);
    let open = open(&head, 2);
    admit(
        &mut image,
        &WorldRecord::OpenAttempt(open.clone()),
        1,
        request,
    )
    .unwrap();
    let occurrence = WorldRecord::Occurrence {
        open: open.clone(),
        relational: WorldRelationalEvidence {
            commit: 1,
            position: 1,
            digest: [9; 32],
        },
        publishing_incarnation: HomeIncarnation::initial(),
    };
    admit(&mut image, &occurrence, 1, request).unwrap();
    image.validate_open_identities(request).unwrap();
    (image, open)
}
pub(super) fn selected(
    image: WorldCheckpointImage,
    request: &ExecutionRequest<'_, '_>,
) -> WorldCheckpointImage {
    let bytes = image.encode(request).unwrap();
    WorldCheckpointImage::decode_selected(bytes.as_ref(), &image.binding, request).unwrap()
}
pub(super) fn wire(
    record: &WorldRecord,
    request: &ExecutionRequest<'_, '_>,
) -> (worth_execution::ExecutionImmutableBytes, [u8; 32]) {
    let bytes = crate::durability::codec::encode(record, request)
        .unwrap()
        .into_immutable_bytes();
    let digest = <[u8; 32]>::from(sha2::Sha256::digest(bytes.as_ref()));
    (bytes, digest)
}
pub(super) fn admit(
    image: &mut WorldCheckpointImage,
    record: &WorldRecord,
    position: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let (bytes, digest) = wire(record, request);
    image.admit_journal_record(record.journal(), position, digest, bytes.as_ref(), request)
}
