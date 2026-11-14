//! Pure decoded leaf ownership and unchanged wire, without invented publication authority.
use super::*;
mod corrective_sources;
mod optional_fields;
use crate::graph_member::{NativeMemberKey, ProjectedMemberCoordinate, ProjectedMemberStorage};
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::schema::data::{SchemaChangeText, SchemaId, SchemaVersionId};
use crate::validation::data::{
    CustomInvariantRuleId, InvariantEntityIds, InvariantPartitionIds, InvariantRelationIds,
};
use std::sync::Arc;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn owned_decode_custom_rule_clone_keeps_only_its_leaf_grant() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let bytes = rmp_serde::to_vec_named("rule-\u{03bb}").unwrap();
    let mut quote = shared_text(BorrowedPayload(&bytes), &request).unwrap();
    add(&mut quote, CustomInvariantRuleId::control_bytes().unwrap()).unwrap();
    let scratch = request.reserve_retained_memory(quote.transient).unwrap();
    let memory = request.reserve_retained_memory(quote.retained).unwrap();
    let actual: CustomInvariantRuleId = rmp_serde::from_slice(&bytes).unwrap();
    actual.retain_decoded_memory(memory).unwrap();
    let retained = actual.clone();
    drop(actual);
    drop(scratch);
    assert_eq!(budget.charged_memory_bytes(), quote.retained);
    assert_eq!(retained.as_str(), "rule-\u{03bb}");
    drop(retained);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn owned_decode_touched_summary_clones_keep_separate_slice_grants() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let entity = EntityId::new(PartitionId(1), 2, 3);
    let relation = RelationId::new(PartitionId(1), 4, 5);
    let entity_bytes = rmp_serde::to_vec_named(&[entity]).unwrap();
    let relation_bytes = rmp_serde::to_vec_named(&[relation]).unwrap();
    let partition_bytes = rmp_serde::to_vec_named(&[PartitionId(1)]).unwrap();
    let mut entity_quote =
        shared_vector::<EntityId>(BorrowedPayload(&entity_bytes), &request, |_| {
            Ok(OwnerAllocation::default())
        })
        .unwrap();
    add(
        &mut entity_quote,
        InvariantEntityIds::control_bytes().unwrap(),
    )
    .unwrap();
    let mut relation_quote =
        shared_vector::<RelationId>(BorrowedPayload(&relation_bytes), &request, |_| {
            Ok(OwnerAllocation::default())
        })
        .unwrap();
    add(
        &mut relation_quote,
        InvariantRelationIds::control_bytes().unwrap(),
    )
    .unwrap();
    let mut partition_quote =
        shared_vector::<PartitionId>(BorrowedPayload(&partition_bytes), &request, |_| {
            Ok(OwnerAllocation::default())
        })
        .unwrap();
    add(
        &mut partition_quote,
        InvariantPartitionIds::control_bytes().unwrap(),
    )
    .unwrap();
    let scratch = request
        .reserve_retained_memory(
            entity_quote.transient + relation_quote.transient + partition_quote.transient,
        )
        .unwrap();
    let entity_memory = request
        .reserve_retained_memory(entity_quote.retained)
        .unwrap();
    let relation_memory = request
        .reserve_retained_memory(relation_quote.retained)
        .unwrap();
    let partition_memory = request
        .reserve_retained_memory(partition_quote.retained)
        .unwrap();
    let entities: InvariantEntityIds = rmp_serde::from_slice(&entity_bytes).unwrap();
    let relations: InvariantRelationIds = rmp_serde::from_slice(&relation_bytes).unwrap();
    let partitions: InvariantPartitionIds = rmp_serde::from_slice(&partition_bytes).unwrap();
    entities.retain_decoded_memory(entity_memory).unwrap();
    relations.retain_decoded_memory(relation_memory).unwrap();
    partitions.retain_decoded_memory(partition_memory).unwrap();
    let retained = relations.clone();
    drop((entities, relations, partitions, scratch));
    assert_eq!(budget.charged_memory_bytes(), relation_quote.retained);
    assert_eq!(retained.as_slice(), &[relation]);
    drop(retained);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn owned_decode_schema_text_and_error_clone_keep_original_leaf_grant() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let bytes = rmp_serde::to_vec_named("schema-\u{03bb}").unwrap();
    let mut quote = shared_text(BorrowedPayload(&bytes), &request).unwrap();
    add(&mut quote, SchemaChangeText::control_bytes().unwrap()).unwrap();
    let scratch = request.reserve_retained_memory(quote.transient).unwrap();
    let memory = request.reserve_retained_memory(quote.retained).unwrap();
    let actual: SchemaChangeText = rmp_serde::from_slice(&bytes).unwrap();
    actual.retain_decoded_memory(memory).unwrap();
    use crate::schema::data::{
        HistoricalInterpretationSensitivity, ProposedSchemaTransition, SchemaDiffAtom,
        SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaPublicationImpact,
        SchemaSubscriberImpact,
    };
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("schema".into()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("schema".into()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef {
                kind: SchemaElementKind::Field,
                schema_id: SchemaId("schema".into()),
                schema_version_id: SchemaVersionId(2),
                kind_id: None,
                element_name: actual,
            },
            Vec::new(),
            SchemaPublicationImpact::None,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::RemovedField {
                field: worth_foundational::FieldKey::new("field").unwrap(),
            },
        )],
    };
    let error = crate::schema::validate_schema_transition(proposed, None).unwrap_err();
    drop(scratch);
    assert_eq!(budget.charged_memory_bytes(), quote.retained);
    assert!(error.detail().contains("schema-\u{03bb}"));
    drop(error);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let (before, after) = optional_fields::schema();
    let (memory, retained) =
        schema::prepare_canonical_readmission(BorrowedPayload(&before), &request).unwrap();
    drop(memory);
    let (memory, omitted_retained) =
        schema::prepare_canonical_readmission(BorrowedPayload(&after), &request).unwrap();
    assert_eq!(retained, omitted_retained);
    drop(memory);
    let (before, after) = optional_fields::validation();
    let (memory, retained) =
        validation::prepare_canonical_readmission(BorrowedPayload(&before), &request).unwrap();
    drop(memory);
    let (memory, omitted_retained) =
        validation::prepare_canonical_readmission(BorrowedPayload(&after), &request).unwrap();
    assert_eq!(retained, omitted_retained);
    drop(memory);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn owned_decode_projected_member_clone_keeps_original_coordinate_grant() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let source = NativeMemberKey::Projected(Arc::new(coordinate()).into());
    let bytes = rmp_serde::to_vec_named(&source).unwrap();
    let quote = validation::member(BorrowedPayload(&bytes), &request).unwrap();
    let scratch = request.reserve_retained_memory(quote.transient).unwrap();
    let memory = request.reserve_retained_memory(quote.retained).unwrap();
    let actual: NativeMemberKey = rmp_serde::from_slice(&bytes).unwrap();
    let NativeMemberKey::Projected(actual) = actual else {
        panic!("projected wire");
    };
    actual.retain_decoded_memory(memory).unwrap();
    let retained = actual.clone();
    drop((actual, scratch));
    assert_eq!(budget.charged_memory_bytes(), quote.retained);
    assert_eq!(retained.as_ref(), &coordinate());
    drop(retained);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn owned_decode_nonmerge_leaf_wire_preserves_prior_named_and_compact_bytes() {
    let rule: Arc<str> = Arc::from("rule-\u{03bb}");
    let entities: Arc<[EntityId]> = Arc::from([EntityId::new(PartitionId(1), 2, 3)]);
    let relations: Arc<[RelationId]> = Arc::from([RelationId::new(PartitionId(1), 4, 5)]);
    let partitions: Arc<[PartitionId]> = Arc::from([PartitionId(1)]);
    let text: Arc<str> = Arc::from("schema-\u{03bb}");
    let member = Arc::new(coordinate());
    let old = (
        rule.clone(),
        entities.clone(),
        relations.clone(),
        partitions.clone(),
        text.clone(),
        member.clone(),
    );
    let actual = (
        CustomInvariantRuleId::new(rule),
        InvariantEntityIds::from(entities),
        InvariantRelationIds::from(relations),
        InvariantPartitionIds::from(partitions),
        SchemaChangeText::from(text),
        ProjectedMemberStorage::from(member),
    );
    let old_named = rmp_serde::to_vec_named(&old).unwrap();
    let old_compact = rmp_serde::to_vec(&old).unwrap();
    assert_eq!(rmp_serde::to_vec_named(&actual).unwrap(), old_named);
    assert_eq!(rmp_serde::to_vec(&actual).unwrap(), old_compact);
    type Leaves = (
        CustomInvariantRuleId,
        InvariantEntityIds,
        InvariantRelationIds,
        InvariantPartitionIds,
        SchemaChangeText,
        ProjectedMemberStorage,
    );
    for bytes in [old_named, old_compact] {
        let decoded: Leaves = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(decoded, actual);
    }
}
fn coordinate() -> ProjectedMemberCoordinate {
    ProjectedMemberCoordinate {
        domain: crate::projected_member_domain::ProjectedMemberDomainIdentity {
            schema: SchemaId("schema".into()),
            version: SchemaVersionId(7),
            name: "domain".into(),
        },
        carrier: EntityId::new(PartitionId(1), 2, 3),
        case: "case-\u{03bb}".into(),
    }
}

#[test]
fn owned_decode_shared_schema_preflight_refuses_zero_original_memory() {
    #[derive(serde::Serialize)]
    struct Payload {
        canonical: Canonical,
    }
    #[derive(serde::Serialize)]
    struct Canonical {
        schema_transition: Transition,
    }
    #[derive(serde::Serialize)]
    struct Transition {
        diff_atoms: Vec<Atom>,
    }
    #[derive(serde::Serialize)]
    struct Atom {
        element: Element,
        detail: Detail,
    }
    #[derive(serde::Serialize)]
    struct Element {
        element_name: &'static str,
    }
    #[derive(serde::Serialize)]
    enum Detail {
        FreeText { detail: &'static str },
    }
    // A selected pure-field grammar for this private preflight, not a fabricated
    // complete artifact/source/receipt or a positive authority-admission fixture.
    let bytes = rmp_serde::to_vec_named(&Payload {
        canonical: Canonical {
            schema_transition: Transition {
                diff_atoms: vec![Atom {
                    element: Element {
                        element_name: "element",
                    },
                    detail: Detail::FreeText { detail: "change" },
                }],
            },
        },
    })
    .unwrap();
    let budget = SerialMemoryBudget::new(Some(0));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(matches!(
        schema::prepare_readmission(&bytes, &request),
        Err(Refusal::Request(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn malformed_stages_vector_is_refused_before_decode() {
    let bytes = corrective_sources::malformed_stages();
    let host = crate::test_execution_policy::serial_request();
    assert!(matches!(
        super::coupled_publication::quote(
            BorrowedPayload(&bytes),
            &ExecutionRequest::serial(&host)
        ),
        Err(Refusal::PayloadDecode { .. })
    ));
}
#[test]
fn unknown_authored_source_variant_still_refuses_noncanonical_payload() {
    let bytes = corrective_sources::unknown_source();
    let host = crate::test_execution_policy::serial_request();
    assert!(matches!(
        super::coupled_publication::quote(
            BorrowedPayload(&bytes),
            &ExecutionRequest::serial(&host)
        ),
        Err(Refusal::NonCanonicalPayload)
    ));
}
#[test]
fn corrective_source_codec_preserves_old_bytes_and_repeated_later_sources() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    for (bytes, expected) in corrective_sources::roundtrips() {
        let quote = super::coupled_publication::quote(BorrowedPayload(&bytes), &request).unwrap();
        let _scratch = request.reserve_retained_memory(quote.transient).unwrap();
        let memory = request.reserve_retained_memory(quote.retained).unwrap();
        let mut decoded: crate::history::data::CanonicalCoupledAuthoredPublication =
            rmp_serde::from_slice(&bytes).unwrap();
        decoded._decoded_memory = Some(Arc::new(memory));
        assert_eq!(decoded.sources, expected);
        assert_eq!(rmp_serde::to_vec_named(&decoded).unwrap(), bytes);
    }
}
#[test]
fn corrective_source_quote_refuses_malformed_empty_missing_or_unknown_fields() {
    let host = crate::test_execution_policy::serial_request();
    for (label, bytes) in corrective_sources::malformed_inputs() {
        assert!(
            super::coupled_publication::quote(
                BorrowedPayload(&bytes),
                &ExecutionRequest::serial(&host)
            )
            .is_err(),
            "{label}"
        );
    }
}
#[test]
fn corrective_owner_union_keeps_missing_extra_and_wrong_predecessor_refusals() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = crate::tests::support::runtime_with_test_schema();
    let source = crate::tests::support::create_entity(&execution, &runtime, "descriptive-owner");
    let original = runtime
        .history
        .positioned_canonical_commits_snapshot()
        .last()
        .unwrap()
        .canonical_arc()
        .clone();
    let admission =
        crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
    execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    crate::history::data::source_scope_closure::assert_exact_owner_refusals(
                        original,
                        admission.basis().descriptor(),
                        source,
                        context,
                    );
                })
        })
        .unwrap()
        .unwrap();
}
