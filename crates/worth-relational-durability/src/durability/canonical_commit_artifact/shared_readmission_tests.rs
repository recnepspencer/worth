//! Actual source readmission retains data funding through shared and weak owners,
//! and a stored owned-graph selection in today's encoding still readmits.

use crate::durability::canonical_commit_artifact::{
    checkpoint_body::readmit_checkpoint_body, frame_encoding::encode_canonical_frame_for_request,
    verified_frame::VerifiedCanonicalCommitFrame as Frame,
};
use branch::history::data::canonical_commit_envelope::control_allocation::CanonicalEnvelopeControlAllocation;
use branch::history::data::canonical_commit_envelope::shared::{
    SharedCanonicalCommitEnvelope, SharedCanonicalCommitEnvelopeOwnerPort as _,
};
use branch::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test as envelope_with_patch_and_update;
use branch::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeFieldsPort as _,
};
use branch::runtime::state::subsystems::history::canonical_publication_routes::position_admission::readmit_positioned_canonical_commit;
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder, SerialMemoryBudget,
    SerialRequest,
};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

#[test]
fn shared_readmitted_envelope_clones_keep_the_original_allocation_and_grant_until_last_drop() {
    let source = source_artifact();
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let artifact = owned_artifact(&source, &request);
    let admitted = artifact.readmitted_body(&request).unwrap();
    let shared = SharedCanonicalCommitEnvelope::from_arc(admitted.canonical_arc().clone());
    drop(admitted);
    drop(artifact);
    let clone = shared.clone();
    assert!(std::ptr::eq(shared.as_ref(), clone.as_ref()));
    let held = budget.charged_memory_bytes();
    assert!(held > fixed_allocation_bill());
    drop(shared);
    assert_eq!(budget.charged_memory_bytes(), held);
    assert_eq!(clone.commit.commit_id, source.commit_id());
    drop(clone);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn actual_weak_position_retains_only_the_fixed_original_control_bill_until_last_drop() {
    let source = source_artifact();
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let artifact = owned_artifact(&source, &request);
    let positioned = artifact.readmitted_body(&request).unwrap();
    let retained = branch::runtime::state::subsystems::history::canonical_publication_routes::retained_position::RetainedCanonicalPosition::from_positioned(&positioned);
    let same = retained.clone();
    let fixed = fixed_allocation_bill();
    drop(artifact);
    assert!(budget.charged_memory_bytes() > fixed);
    drop(positioned);
    assert!(retained.resident().is_none());
    assert_eq!(
        budget.charged_memory_bytes(),
        fixed,
        "payload heaps and original raw bytes are gone"
    );
    drop(retained);
    assert_eq!(
        budget.charged_memory_bytes(),
        fixed,
        "the remaining actual Weak owner shares one bill"
    );
    drop(same);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

fn owned_artifact(source: &Frame, request: &ExecutionRequest<'_, '_>) -> Frame {
    let mut bytes =
        ExecutionRequestArrayBuilder::<u8>::allocate(*request, source.as_bytes().len()).unwrap();
    bytes.extend_from_slice(source.as_bytes()).unwrap();
    Frame::from_owned_bytes(bytes.seal().unwrap().into_immutable_bytes()).unwrap()
}

fn canonical_source() -> CanonicalCommitEnvelope {
    let id = EntityId::new(PartitionId::main(), 1, 1);
    envelope_with_patch_and_update(RecordRef::Entity(id), id)
}

fn source_artifact() -> Frame {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let request = ExecutionRequest::serial(&host);
    let envelope = canonical_source();
    assert!((*envelope.control_allocation_for_test()).is_none());
    let positioned =
        readmit_positioned_canonical_commit(PatchStreamPosition(1), Arc::new(envelope)).unwrap();
    encode_canonical_frame_for_request(&positioned, &request).unwrap()
}

fn fixed_allocation_bill() -> u64 {
    // Independent oracle over the concrete allocated types, not the production
    // quote helper: a zero/omitted control reservation must fail this proof.
    fn bytes<T>() -> usize {
        std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<T>())
            .unwrap()
            .0
            .pad_to_align()
            .size()
    }
    let bytes =
        bytes::<branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope>()
            .checked_add(bytes::<CanonicalEnvelopeControlAllocation>())
            .unwrap();
    assert!(bytes > 0);
    u64::try_from(bytes).unwrap()
}

#[test]
fn embedded_checkpoint_preserves_ignored_unknown_nested_fields_without_retaining_their_heaps() {
    use branch::history::data::EmbeddedCheckpointEnvelopeRef;
    #[derive(serde::Serialize)]
    struct Extended<'a, T> {
        #[serde(flatten)]
        original: EmbeddedCheckpointEnvelopeRef<'a>,
        #[serde(rename = "future \" \\ λ")]
        future: &'a T,
    }
    let original = canonical_source();
    let original = &original;
    assert!(original.derived_index_artifacts.is_empty());
    // Fixture input uses the canonical owner's wire projection; no copied
    // canonical projector or JSON conversion of its u128 values participates.
    let known = rmp_serde::to_vec_named(&EmbeddedCheckpointEnvelopeRef::new(original)).unwrap();
    let future = serde_json::json!({"escaped \" \\ λ": ["line\n\u{0000}雪".repeat(1024), {"inner": [true, null, 7]}]});
    let extended = rmp_serde::to_vec_named(&Extended {
        original: EmbeddedCheckpointEnvelopeRef::new(original),
        future: &future,
    })
    .unwrap();
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let baseline = readmit_checkpoint_body(&known, None, &request).unwrap();
    assert_eq!(&baseline, original);
    let held = budget.charged_memory_bytes();
    assert!(held > 0);
    drop(baseline);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let check = |bytes: &[u8]| {
        let accepted = readmit_checkpoint_body(bytes, None, &request).unwrap();
        assert_eq!(&accepted, original);
        assert_eq!(
            budget.charged_memory_bytes(),
            held,
            "ignored/absent input is not retained decoded storage"
        );
        drop(accepted);
        assert_eq!(budget.charged_memory_bytes(), 0);
    };
    check(&extended);
    // Use rmp-serde's actual extension producer, not a copied envelope codec.
    #[derive(serde::Serialize)]
    #[serde(rename = "_ExtStruct")]
    struct Extension<'a>((i8, Binary<'a>));
    struct Binary<'a>(&'a [u8]);
    impl serde::Serialize for Binary<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(self.0)
        }
    }
    for (length, marker) in [
        (1, 0xd4),
        (2, 0xd5),
        (4, 0xd6),
        (8, 0xd7),
        (16, 0xd8),
        (3, 0xc7),
        (256, 0xc8),
        (65536, 0xc9),
    ] {
        let payload = vec![0x7b; length];
        let extension = Extension((-7, Binary(&payload)));
        assert_eq!(rmp_serde::to_vec_named(&extension).unwrap()[0], marker);
        let bytes = rmp_serde::to_vec_named(&Extended {
            original: EmbeddedCheckpointEnvelopeRef::new(original),
            future: &extension,
        })
        .unwrap();
        check(&bytes);
    }
    // Remove only original nil Option fields from real producer bytes. The
    // unrelated u128 tokens and every known body lane remain untouched.
    let mut omitted = known.clone();
    assert_eq!(omitted[0], 0xde, "the actual producer emits a map16");
    let mut fields = u16::from_be_bytes([omitted[1], omitted[2]]);
    for name in [
        "schema_transition",
        "caller_association",
        "required_output_origin",
        "required_output_materialization",
        "branch_cell_checkpoint",
        "strategy_artifacts",
        "merge_execution_authority",
        "schema_continuation_descriptor",
        "schema_reconciliation_descriptor",
    ] {
        let mut needle = rmp_serde::to_vec(name).unwrap();
        needle.push(0xc0);
        let matches: Vec<_> = omitted
            .windows(needle.len())
            .enumerate()
            .filter_map(|(at, bytes)| (bytes == needle).then_some(at))
            .collect();
        if name == "schema_transition" {
            assert_eq!(matches.len(), 1);
        }
        if let [at] = matches.as_slice() {
            omitted.drain(*at..*at + needle.len());
            fields -= 1;
            omitted[1..3].copy_from_slice(&fields.to_be_bytes());
            check(&omitted);
        }
    }
}

/// A stored owned-graph selection in today's encoding (full current and
/// selected sets, no encoding tag) stays readable byte-exact: readmission
/// re-encodes it to the same payload, and its targets are the whole roster
/// although only the model changed. S5's tagged encoding must keep this.
#[test]
fn a_v1_navigation_selection_readmits_byte_exact() {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
    use branch::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelectionOwnerPort as _;
    use branch::history::data::canonical_commit_envelope::CanonicalCommitAuthorityKind;
    use std::collections::BTreeSet;
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-relational/assets/fixtures/owned_graph_selection_v1.frame"
    ));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let readmitted = RelationalCanonicalCommitArtifact::readmit(bytes, &request).unwrap();
    assert_eq!(readmitted.commit_id().0, 23);
    let CanonicalCommitAuthorityKind::OwnedGraphSelection(selection) =
        readmitted.envelope().authority_kind()
    else {
        panic!("the fixture is an owned-graph selection");
    };
    let targets: BTreeSet<_> = selection.targets().collect();
    let entities: Vec<_> = targets
        .iter()
        .filter_map(|target| match target {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .collect();
    assert_eq!(targets.len() - entities.len(), 10, "every owning edge");
    assert!(entities.contains(&selection.model()));
    assert_eq!(
        entities
            .iter()
            .filter(|id| id.partition_id == PartitionId(3))
            .count(),
        10,
        "every member, not only the edited model"
    );
    assert_eq!(entities.len(), 11);
}
