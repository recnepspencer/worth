//! Actual source readmission retains data funding through shared and weak owners.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::history::data::{CanonicalEnvelopeControlAllocation, SharedCanonicalCommitEnvelope};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder, SerialMemoryBudget,
    SerialRequest,
};

#[test]
fn shared_readmitted_envelope_clones_keep_the_original_allocation_and_grant_until_last_drop() {
    let source = source_artifact();
    let budget = SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let artifact = owned_artifact(&source, &request);
    let admitted = artifact.readmit_framed(&request).unwrap();
    let shared =
        SharedCanonicalCommitEnvelope::from_arc(admitted.into_positioned().canonical_arc().clone());
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
    let positioned = artifact.readmit_framed(&request).unwrap().into_positioned();
    let retained = crate::runtime::RetainedCanonicalPosition::from_positioned(&positioned);
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

fn owned_artifact(source: &Artifact, request: &ExecutionRequest<'_, '_>) -> Artifact {
    let mut bytes =
        ExecutionRequestArrayBuilder::<u8>::allocate(*request, source.as_bytes().len()).unwrap();
    bytes.extend_from_slice(source.as_bytes()).unwrap();
    Artifact::from_owned_bytes(bytes.seal().unwrap().into_immutable_bytes()).unwrap()
}

fn source_artifact() -> Artifact {
    // Ordinary source production precedes the bounded readmission operation.
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = crate::tests::support::durable_runtime_with_test_schema();
    let result = crate::tests::support::create_entity_outcome(
        &request,
        &runtime,
        "original-shared-readmission-heap",
    );
    let positioned = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .pop()
        .unwrap();
    assert!(
        positioned.envelope().control_allocation.is_none(),
        "ordinary live publication adds no control Arc"
    );
    let artifact = Artifact::from_positioned(
        &positioned,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    crate::tests::support::release_test_commit_snapshot(&runtime, &result);
    artifact
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
    let bytes = bytes::<crate::history::data::CanonicalCommitEnvelope>()
        .checked_add(bytes::<CanonicalEnvelopeControlAllocation>())
        .unwrap();
    assert!(bytes > 0);
    u64::try_from(bytes).unwrap()
}

#[test]
fn embedded_checkpoint_preserves_ignored_unknown_nested_fields_without_retaining_their_heaps() {
    use crate::history::data::EmbeddedCheckpointEnvelopeRef;
    #[derive(serde::Serialize)]
    struct Extended<'a, T> {
        #[serde(flatten)]
        original: EmbeddedCheckpointEnvelopeRef<'a>,
        #[serde(rename = "future \" \\ λ")]
        future: &'a T,
    }
    let producer = crate::test_execution_policy::serial_request();
    let produce = ExecutionRequest::serial(&producer);
    let runtime = crate::tests::support::durable_runtime_with_test_schema();
    let result =
        crate::tests::support::create_entity_outcome(&produce, &runtime, "known-original-cp-body");
    let original = result.publication().envelope.as_ref();
    assert!(original.derived_index_artifacts.is_empty());
    // Ordinary input preparation uses the real producer projection; no copied
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
    let baseline = Artifact::readmit_checkpoint_body(&known, None, &request).unwrap();
    assert_eq!(&baseline, original);
    let held = budget.charged_memory_bytes();
    assert!(held > 0);
    drop(baseline);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let check = |bytes: &[u8]| {
        let accepted = Artifact::readmit_checkpoint_body(bytes, None, &request).unwrap();
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
    crate::tests::support::release_test_commit_snapshot(&runtime, &result);
}
