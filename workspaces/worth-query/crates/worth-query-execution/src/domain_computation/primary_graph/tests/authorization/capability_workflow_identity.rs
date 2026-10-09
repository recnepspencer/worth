//! A capability workflow request, composed through the doors Publication's
//! capability workflow entries drive: its input is encoded once and governs
//! both the capability admission and the intent, and its key is encoded once
//! with the principal and scoped to the operation.
//!
//! Only Publication may hold the publication access that reports the key's
//! work into the admission, so these proofs admit through the crate's own
//! encoded-input door and check the two derivations separately. Bank's
//! delegation receipts pin their sum through the real entry.

use serde::Serialize;
use worth_query_declaration::facade::application_operation::ApplicationEncodedInput;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::capability::{
    CapabilityTouchOperationInputBinding, ComposedCapabilityTouchOperation,
    ComposedCapabilityTouchOperationInputBinding,
};
use super::super::fixture::{
    installed_capability_authorization_world, live_scope, CapabilityTouchInput,
    CapabilityTouchOperation, IdentityExecutionSchema, TouchAccountCapability,
};

use super::capability_progression::{build_touch_program, capability_input, time};
use crate::domain_computation::authorization::admit_encoded_capability_access;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryCapabilityWorkflowIdempotency;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

thread_local! {
    static KEY_ENCODINGS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// A client key that counts its encodings on this thread.
struct CountingKey(&'static str);

impl Serialize for CountingKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        KEY_ENCODINGS.with(|count| count.set(count.get() + 1));
        serializer.serialize_newtype_struct("CountingKey", self.0)
    }
}

fn reset_encoding_counts() {
    CapabilityTouchInput::reset_encoding_count();
    KEY_ENCODINGS.with(|count| count.set(0));
}

fn encoding_counts() -> (u32, u32) {
    (
        CapabilityTouchInput::encoding_count(),
        KEY_ENCODINGS.with(std::cell::Cell::get),
    )
}

fn bind_touch(
    key: &'static str,
    input: &ApplicationEncodedInput<CapabilityTouchOperationInputBinding>,
    principal: u64,
) -> WorthQueryCapabilityWorkflowIdempotency {
    WorthQueryCapabilityWorkflowIdempotency::bind::<
        IdentityExecutionSchema,
        CapabilityTouchOperation,
        _,
        u64,
        U64ApplicationValueBinding,
    >(&CountingKey(key), input, &principal)
    .expect("the workflow key encodes")
}

#[test]
fn a_capability_workflow_encodes_its_input_and_key_once_each_first_and_replayed() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100); 24]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let schema = world.application.installed_schema();
    let capability = schema
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let operation = schema
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();
    let prepare = || {
        reset_encoding_counts();
        let input = ApplicationEncodedInput::<CapabilityTouchOperationInputBinding>::encode(
            capability_input(100),
        )
        .expect("the input encodes");
        let workflow = bind_touch("workflow-key", &input, *principal.principal_identity());
        let selected = world.selected_product();
        let access = admit_encoded_capability_access(
            &world.application,
            selected.product(),
            &principal,
            &capability,
            input,
            &request,
        )
        .expect("the encoded input is admitted");
        let admission_work = access.admission_canonical_work();
        let admission = world
            .application
            .authorize_capability_operation(access, &operation, Default::default())
            .unwrap();
        let program = build_touch_program(&world, &request, admission, "workflow");
        assert_eq!(encoding_counts(), (1, 1), "one input and one key encoding");
        (program, workflow, admission_work)
    };
    let (first, first_workflow, first_work) = prepare();
    let (retry, retry_workflow, retry_work) = prepare();
    assert_eq!(
        first_workflow, retry_workflow,
        "the same request binds the same way"
    );
    assert_input_work(first_work);
    assert_key_work(WorthQueryCanonicalWorkEvidence::streamed_identities(
        first_workflow.key_work(),
    ));
    assert_eq!(retry_work, first_work);

    reset_encoding_counts();
    let WorthQueryApplicationCommitOutcome::Committed(committed) =
        world.application.compare_and_commit_application(
            first,
            first_workflow.binding(),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the first capability workflow request commits");
    };
    assert_eq!(committed.canonical_work().admission(), first_work);
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            retry_workflow.binding(),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the unchanged retry replays the commit");
    };
    assert_eq!(recovered.canonical_work().admission(), first_work);
    assert_eq!(
        encoding_counts(),
        (0, 0),
        "committing encodes nothing again"
    );
}

/// The governed input, encoded once: 289 encoded bytes, hashed with 88
/// bytes of framing into 377, 7 blocks.
fn assert_input_work(work: WorthQueryCanonicalWorkEvidence) {
    assert_one_derivation(work, 289, 377, 7);
}

/// The key over the principal basis and "workflow-key": 92 encoded bytes,
/// hashed with 78 bytes of framing (the 38-byte capability workflow key domain
/// and the 24-byte operation identifier, each behind an 8-byte length) into
/// 170, 3 blocks. Publication adds it to the input's work in admission.
fn assert_key_work(work: WorthQueryCanonicalWorkEvidence) {
    assert_one_derivation(work, 92, 170, 3);
}

fn assert_one_derivation(
    work: WorthQueryCanonicalWorkEvidence,
    encoded: usize,
    hashed: usize,
    blocks: usize,
) {
    assert_eq!(work.basis_preparations(), 0);
    assert_eq!(work.digest_derivations(), 1);
    assert_eq!(work.canonical_entries(), 1);
    assert_eq!(work.canonical_encoded_bytes(), encoded);
    assert_eq!(work.canonical_material_allocation_bytes(), 0);
    assert_eq!(work.sha256_input_bytes(), hashed);
    assert_eq!(work.sha256_compression_blocks(), blocks);
    assert_eq!(work.digest_text_materializations(), 0);
}

#[test]
fn a_capability_workflow_key_never_replays_across_principals_or_operations() {
    let touch = ApplicationEncodedInput::<CapabilityTouchOperationInputBinding>::encode(
        capability_input(100),
    )
    .expect("the input encodes");
    let composed = ApplicationEncodedInput::<ComposedCapabilityTouchOperationInputBinding>::encode(
        capability_input(100),
    )
    .expect("the input encodes");
    let original = bind_touch("shared-key", &touch, 1);
    let under_composed = WorthQueryCapabilityWorkflowIdempotency::bind::<
        IdentityExecutionSchema,
        ComposedCapabilityTouchOperation,
        _,
        u64,
        U64ApplicationValueBinding,
    >(&CountingKey("shared-key"), &composed, &1)
    .expect("the workflow key encodes")
    .binding();

    assert_eq!(
        original,
        bind_touch("shared-key", &touch, 1),
        "a retry replays"
    );
    assert_ne!(
        original.binding().key_identity(),
        bind_touch("shared-key", &touch, 2).binding().key_identity(),
        "the same key from another principal is another key"
    );
    assert_eq!(
        original.binding().intent_identity(),
        under_composed.intent_identity(),
        "both operations govern the same input identity"
    );
    assert_ne!(
        original.binding().key_identity(),
        under_composed.key_identity(),
        "the same key under another operation is another key"
    );
}
