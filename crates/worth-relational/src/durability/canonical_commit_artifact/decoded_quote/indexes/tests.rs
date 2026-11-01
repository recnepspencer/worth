//! Existing index backings carry their own original admission through clones.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn owned_decode_index_backings_keep_only_their_original_grants() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let (delta_bytes, writes_bytes) = input();
    let delta_quote = delta(BorrowedPayload(&delta_bytes), &request).unwrap();
    let writes_quote = authored(BorrowedPayload(&writes_bytes), &request).unwrap();
    let scratch = request
        .reserve_retained_memory(delta_quote.transient + writes_quote.transient)
        .unwrap();
    let delta_memory = request
        .reserve_retained_memory(delta_quote.retained)
        .unwrap();
    let writes_memory = request
        .reserve_retained_memory(writes_quote.retained)
        .unwrap();
    let mut decoded_delta: CanonicalIndexDefinitionDelta =
        rmp_serde::from_slice(&delta_bytes).unwrap();
    let mut decoded_writes: CanonicalAuthoredIndexWrites =
        rmp_serde::from_slice(&writes_bytes).unwrap();
    decoded_delta.retain_decoded_memory(delta_memory).unwrap();
    decoded_writes.retain_decoded_memory(writes_memory).unwrap();
    // A parsed Complete roster does not acquire private live coverage.
    assert!(decoded_writes.coverage().is_none());
    assert!(decoded_writes.verified_entries().is_none());
    let retained_delta = decoded_delta.clone();
    let retained_writes = decoded_writes.clone();
    drop((decoded_delta, decoded_writes, scratch));
    assert_eq!(
        budget.charged_memory_bytes(),
        delta_quote.retained + writes_quote.retained
    );
    assert_eq!(retained_delta.definitions()[0].name, "index-\u{03bb}");
    drop(retained_delta);
    assert_eq!(budget.charged_memory_bytes(), writes_quote.retained);
    assert!(retained_writes.coverage().is_none());
    drop(retained_writes);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn owned_decode_index_predecode_denial_and_alias_keep_original_grant() {
    let cancellation = CancellationSource::new();
    let (delta_bytes, writes_bytes) = input();
    let budget = SerialMemoryBudget::new(Some(0));
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let delta_quote = delta(BorrowedPayload(&delta_bytes), &request).unwrap();
    let writes_quote = authored(BorrowedPayload(&writes_bytes), &request).unwrap();
    let quote = DecodedQuote {
        index_delta: delta_quote,
        index_writes: writes_quote,
        ..DecodedQuote::default()
    };
    assert!(matches!(
        quote.admit(&request, None),
        Err(Refusal::Request(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);

    let budget = SerialMemoryBudget::new(Some(65_536));
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let scratch = request
        .reserve_retained_memory(delta_quote.transient)
        .unwrap();
    let memory = request
        .reserve_retained_memory(delta_quote.retained)
        .unwrap();
    let mut decoded: CanonicalIndexDefinitionDelta = rmp_serde::from_slice(&delta_bytes).unwrap();
    let alias = decoded.clone();
    let memory = decoded
        .retain_decoded_memory(memory)
        .expect_err("actual alias prevents mutable backing handoff");
    drop(alias);
    assert_eq!(memory.as_ref().unwrap().bytes(), delta_quote.retained);
    decoded.retain_decoded_memory(memory).unwrap();
    drop(scratch);
    assert_eq!(budget.charged_memory_bytes(), delta_quote.retained);
    let unused = request
        .reserve_retained_memory(delta_quote.retained)
        .unwrap();
    let unused = decoded
        .retain_decoded_memory(unused)
        .expect_err("existing original slot cannot be overwritten");
    assert_eq!(unused.as_ref().unwrap().bytes(), delta_quote.retained);
    drop(unused);
    assert_eq!(budget.charged_memory_bytes(), delta_quote.retained);
    drop(decoded);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

fn input() -> (Vec<u8>, Vec<u8>) {
    let definition = DerivedIndexDefinition {
        index_id: crate::indexes::data::DerivedIndexId(1),
        name: "index-\u{03bb}".into(),
        kind: crate::indexes::data::DerivedIndexKind::EntityField {
            field_locator: crate::transactions::data::planned_single_field_locator(
                worth_foundational::facade::AspectKey::new("aspect").unwrap(),
                worth_foundational::facade::FieldKey::new("field").unwrap(),
            ),
        },
        branch_scoped: true,
    };
    let delta = CanonicalIndexDefinitionDelta::prepared(1, 2, vec![definition], None);
    #[derive(serde::Serialize)]
    enum ParsedEvidence {
        Complete(Vec<CanonicalAuthoredIndexWrite>),
    }
    // Pure exact cold field grammar; this never calls the private live issuer
    // or invents a catalog incarnation/coverage coordinate.
    (
        rmp_serde::to_vec_named(&delta).unwrap(),
        rmp_serde::to_vec_named(&ParsedEvidence::Complete(Vec::new())).unwrap(),
    )
}
