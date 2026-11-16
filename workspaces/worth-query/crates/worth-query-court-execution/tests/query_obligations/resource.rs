//! DATA guard: retain original inputs/ingress without treating absent owners as release.
use worth_foundational::facade::{AspectValue, HomeIdentity, OwnedAsyncObligationIdentity};
use worth_query_execution::facade::query_obligations::*;
use worth_relational::facade::identity::EntityId;
fn append(
    ledger: &mut QueryObligationLedger,
    generations: &mut QueryObligationPublicationGenerations,
    record: QueryResourceRecord,
) -> Vec<u8> {
    let frame = ledger.prepare(record, generations).unwrap();
    let bytes = frame.bytes().to_vec();
    ledger
        .readmit(QueryObligationFrame::readmit(&bytes).unwrap())
        .unwrap();
    bytes
}
#[test]
fn resource_owner_data_keeps_original_input_ingress_and_refuses_unrecorded_release() {
    let home = HomeIdentity::from_observed_bytes([71; 16]).unwrap();
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let identity = OwnedAsyncObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let actor: EntityId =
        serde_json::from_str(r#"{"partition_id":0,"local_slot":1,"generation":0}"#).unwrap();
    let original = QueryResourceOriginal {
        binding: "declared resource".to_owned(),
        package: [1; 32],
        schema: [2; 32],
        branch: "original branch".to_owned(),
        commit: 19,
        principal: actor,
        scope_subject: actor,
        scope_aspect: "scope".to_owned(),
        scope_fields: vec!["key".to_owned()],
        scope_value: AspectValue::UInt64(7),
        query: [3; 32],
        parameters: vec![("p".to_owned(), AspectValue::UInt64(11))],
        input: AspectValue::UInt64(41),
        declaration: QueryOwnedAsyncDefinition {
            identity: "installed source".to_owned(),
            payload_contract: 5,
            max_payload_bytes: 1024,
            retry_max_attempts: 3,
            retry_delay_ticks: 2,
            timeout_ticks: 9,
            managed_queue_capacity: std::num::NonZeroU64::new(2),
            retention: QueryOwnedAsyncRetention::ProcessLocal,
        },
    };
    let retain = append(
        &mut ledger,
        &mut generations,
        QueryResourceRecord::Retain(QueryResourceObligation {
            identity,
            original: original.clone(),
            handoff: QueryResourceHandoff::default(),
        }),
    );
    let ingress = vec![
        b"first unverified frame".to_vec(),
        b"second unverified frame".to_vec(),
    ];
    let handoff = QueryResourceHandoff {
        ingress: ingress.clone(),
        result: Some((QueryResourceResultKind::Result, AspectValue::UInt64(123))),
        ..Default::default()
    };
    let pending = append(
        &mut ledger,
        &mut generations,
        QueryResourceRecord::Handoff {
            identity,
            handoff: handoff.clone(),
        },
    );
    let mut cold = QueryObligationLedger::empty(home);
    for bytes in [retain, pending] {
        cold.readmit(QueryObligationFrame::readmit(&bytes).unwrap())
            .unwrap();
    }
    let row = cold.resources().next().unwrap();
    assert_eq!(row.identity, identity);
    assert_eq!(row.original, original);
    assert_eq!(row.handoff.ingress, ingress);
    assert_eq!(
        row.handoff.result,
        Some((QueryResourceResultKind::Result, AspectValue::UInt64(123))),
        "only the independently committed result is retained; ingress bytes are not promoted"
    );
    let mut generations = QueryObligationPublicationGenerations::from_journal(&cold);
    assert_eq!(
        cold.prepare(
            QueryResourceRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::OwnedAsyncPending { identity })
    );
    // No Native admission occurred: only an explicit original no-request release
    // can discharge. This is DATA validation, not proof of physical cleanup.
    let handoff = QueryResourceHandoff {
        release: Some(QueryResourceRelease {
            terminal: None,
            predecessor: None,
        }),
        ..handoff
    };
    append(
        &mut cold,
        &mut generations,
        QueryResourceRecord::Handoff { identity, handoff },
    );
    append(
        &mut cold,
        &mut generations,
        QueryResourceRecord::Discharge { identity },
    );
    assert!(cold.resources().next().is_none());
}
