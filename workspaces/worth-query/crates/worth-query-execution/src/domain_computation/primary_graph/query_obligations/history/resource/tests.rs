//! DATA cutover guard only: no physical publication or resource authority is forged.
use super::*;
use worth_foundational::facade::{AspectValue, HomeIdentity};
use worth_relational::facade::identity::EntityId;
#[test]
fn resource_disposal_resume_consumes_only_its_acknowledged_original_discharge() {
    let home = HomeIdentity::from_observed_bytes([80; 16]).unwrap();
    let identity = OwnedAsyncObligationIdentity::from_recorded(home, std::num::NonZeroU64::MIN);
    let mut journal = QueryObligationJournalState::from_ledger(QueryObligationLedger::empty(home));
    let subject: EntityId =
        serde_json::from_str(r#"{"partition_id":0,"local_slot":1,"generation":0}"#).unwrap();
    let original = QueryResourceOriginal {
        binding: "resource".into(),
        package: [1; 32],
        schema: [2; 32],
        branch: "main".into(),
        commit: 1,
        principal: subject,
        scope_subject: subject,
        scope_aspect: "subject".into(),
        scope_fields: vec!["key".into()],
        scope_value: AspectValue::UInt64(7),
        query: [3; 32],
        parameters: Vec::new(),
        input: AspectValue::UInt64(41),
        declaration: worth_runtime_bridge::facade::BridgeOwnedAsyncRequestResponseDeclaration::new(
            "source", 1, 64, 1, 1, 1,
        )
        .with_managed_queue_capacity(std::num::NonZeroU64::MIN)
        .with_process_local_retention()
        .description(),
    };
    for record in [
        QueryResourceRecord::Retain(QueryResourceObligation {
            identity,
            original,
            handoff: QueryResourceHandoff::default(),
        }),
        QueryResourceRecord::Handoff {
            identity,
            handoff: QueryResourceHandoff {
                release: Some(QueryResourceRelease {
                    terminal: None,
                    predecessor: None,
                }),
                ..Default::default()
            },
        },
    ] {
        journal.prepare_pending(record).unwrap();
        journal
            .ledger
            .readmit(journal.pending.take().unwrap())
            .unwrap();
    }
    journal
        .prepare_pending(QueryResourceRecord::Discharge { identity })
        .unwrap();
    let mut disposal = ResourceDisposal::Discharging;
    // Pending DATA is not an acknowledged discharge: the original remains.
    disposal
        .resume_after_settlement(journal.ledger.resources.contains_key(&identity))
        .unwrap();
    assert_eq!(disposal, ResourceDisposal::Discharging);
    // Model only the shared driver's admitted DATA cutover, not physical IO.
    journal
        .ledger
        .readmit(journal.pending.take().unwrap())
        .unwrap();
    let acknowledged = journal.ledger.position();
    disposal
        .resume_after_settlement(journal.ledger.resources.contains_key(&identity))
        .unwrap();
    assert_eq!(disposal, ResourceDisposal::Discharged);
    disposal.resume_after_settlement(false).unwrap();
    assert_eq!(
        journal.ledger.position(),
        acknowledged,
        "resume requires no new handoff or replacement work"
    );
    let mut never_acknowledged = ResourceDisposal::Retained;
    assert_eq!(
        never_acknowledged.resume_after_settlement(false),
        Err(QueryObligationDenial::UnknownObligation)
    );
    assert_eq!(
        never_acknowledged,
        ResourceDisposal::Retained,
        "an acquire's no-effect is not disposal acknowledgment"
    );
}
