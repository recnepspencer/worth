//! Value-level grammar and fold laws; these tests issue no original authority.
use super::*;
use std::num::NonZeroU64;
use worth_foundational::facade::{
    DurableWorldOwnerIdentity, RetentionClass, WorldOwnerSlotIdentity,
};
use worth_relational::facade::{
    history::{BranchId, CommitId},
    identity::{EntityId, PartitionId, VersionId},
};

fn original(home: HomeIdentity, ordinal: u64) -> Original {
    Original::from_recorded(
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1)),
        ordinal,
    )
}
fn catalog() -> MaterialCatalog {
    MaterialCatalog::empty(HomeIdentity::from_observed_bytes([1; 16]).unwrap())
}
fn entity() -> EntityId {
    EntityId::new(PartitionId::main(), 7, 1)
}
fn scope() -> RecordedScope {
    RecordedScope {
        location: RecordedLocation {
            aspect: "account".into(),
            fields: vec!["name".into()],
        },
        value: worth_foundational::facade::AspectValue::UInt64(7),
        subject: entity(),
    }
}
fn capture(ledger: &MaterialCatalog) -> MaterialRecord {
    MaterialRecord::Capture {
        material: Box::new(MaterialCapture {
            original: original(ledger.home, 1),
            program: [2; 32],
            mutation: "edit".into(),
            key: [4; 32],
            intent: "recorded exact intent".into(),
            intent_identity: [4; 32],
            input_identity: [5; 32],
            input: vec![1, 2, 3],
            principal: entity(),
            scope_entity: entity(),
            scope: Some(scope()),
            disclosure: None,
            preimage: Some(Vec::new()),
            changed_fields: None,
            content: Default::default(),
        }),
    }
}
fn receipt() -> worth_relational::facade::history::RelationalCommitReceipt {
    worth_relational::facade::history::RelationalCommitReceipt {
        commit_id: CommitId(11),
        version_id: VersionId(11),
        branch_id: BranchId("main".into()),
        parents: vec![CommitId(10)],
    }
}
fn ready(ledger: &MaterialCatalog) -> MaterialRecord {
    MaterialRecord::Ready {
        material: Box::new(MaterialReady {
            original: original(ledger.home, 1),
            commit: receipt(),
            roles: Vec::new(),
            output_fields: Vec::new(),
            content: Default::default(),
            source_facts: RecordedSourceFacts::Empty,
        }),
    }
}
fn append(ledger: &mut MaterialCatalog, record: MaterialRecord) -> QueryCorrectionMaterialFrame {
    let frame = ledger
        .frame(NonZeroU64::new(ledger.generation + 1).unwrap(), &record)
        .unwrap();
    ledger.fold(&frame).unwrap();
    frame
}
fn available() -> MaterialCatalog {
    let mut ledger = catalog();
    let first = capture(&ledger);
    append(&mut ledger, first);
    let next = ready(&ledger);
    append(&mut ledger, next);
    ledger
}
fn prepared(ledger: &MaterialCatalog) -> PreparedCorrectionSnapshot {
    PreparedCorrectionSnapshot {
        key: [6; 32],
        intent: [7; 32],
        action: "undo".into(),
        input: vec![8, 9],
        input_identity: [10; 32],
        actor: entity(),
        scope: scope(),
        target: RecordedCorrectionTarget {
            owner: original(ledger.home, 1).owner(),
            branch: "main".into(),
            incarnation: 1,
            generation: 2,
            commit: 3,
            relational: receipt(),
            program: [2; 32],
        },
    }
}
fn pin(ledger: &MaterialCatalog) -> MaterialRecord {
    MaterialRecord::Pin {
        identity: CorrectionMaterialPinIdentity::from_recorded(
            ledger.home,
            NonZeroU64::new(ledger.position + 1).unwrap(),
        ),
        original: original(ledger.home, 1),
        owner: CorrectionMaterialPinOwner::PreparedCommand { key: [6; 32] },
        prepared: Box::new(prepared(ledger)),
        class: RetentionClass::MandatoryObligation,
    }
}

#[test]
fn recorded_material_round_trips_before_it_becomes_available() {
    let mut ledger = catalog();
    let record = capture(&ledger);
    let frame = append(&mut ledger, record.clone());
    assert_eq!(MaterialRecord::decode(&frame).unwrap(), record);
    assert_eq!(
        frame.body_retention_class(),
        RetentionClass::MandatoryObligation
    );
    assert_eq!(
        ledger.material[&original(ledger.home, 1)]
            .available()
            .unwrap_err(),
        Denial::MaterialNotReady
    );
    let record = ready(&ledger);
    let frame = append(&mut ledger, record);
    assert_eq!(
        frame.body_retention_class(),
        RetentionClass::OptionalRetainedMaterial
    );
    assert_eq!(
        ledger.material[&original(ledger.home, 1)]
            .available()
            .unwrap()
            .1
            .commit,
        receipt()
    );
}
#[test]
fn a_body_class_mismatch_cannot_advance_the_semantic_prefix() {
    let mut ledger = catalog();
    let record = capture(&ledger);
    let frame = QueryCorrectionMaterialFrame::prepare(
        ledger.home,
        NonZeroU64::new(1).unwrap(),
        [0; 32],
        NonZeroU64::new(1).unwrap(),
        RetentionClass::OptionalRetainedMaterial,
        &record.encode().unwrap(),
    )
    .unwrap();
    assert_eq!(ledger.fold(&frame), Err(Denial::RetentionClassMismatch));
    assert_eq!(ledger.position, 0);
    assert!(ledger.material.is_empty());
}
#[test]
fn owning_discharge_preserves_the_terminal_command_through_cold_fold() {
    let mut ledger = catalog();
    let mut frames = Vec::new();
    let record = capture(&ledger);
    frames.push(append(&mut ledger, record));
    let record = ready(&ledger);
    frames.push(append(&mut ledger, record));
    let record = pin(&ledger);
    frames.push(append(&mut ledger, record));
    let (identity, row) = ledger.pins.first_key_value().unwrap();
    let identity = *identity;
    let command = Arc::clone(row.prepared().unwrap());
    assert_eq!(ledger.pin_count(original(ledger.home, 1)), 1);
    assert_eq!(*command, prepared(&ledger));
    frames.push(append(
        &mut ledger,
        MaterialRecord::DischargePreparation {
            identity,
            key: command.key,
            class: RetentionClass::OptionalRetainedMaterial,
        },
    ));
    let mut cold = catalog();
    for frame in frames {
        cold.fold(&QueryCorrectionMaterialFrame::readmit(frame.bytes().to_vec()).unwrap())
            .unwrap();
    }
    assert_eq!(cold.pin_count(original(cold.home, 1)), 0);
    assert!(!cold.pins.contains_key(&identity));
    assert_eq!(
        cold.discharged[&command.key],
        (original(cold.home, 1), command.intent)
    );
    let record = pin(&cold);
    assert_eq!(
        cold.frame(NonZeroU64::new(cold.position + 1).unwrap(), &record),
        Err(Denial::PreparationDischarged)
    );
}
#[test]
fn pin_owners_cannot_be_replaced_without_the_prepared_to_publication_handoff() {
    let mut ledger = available();
    let record = pin(&ledger);
    append(&mut ledger, record);
    let (identity, row) = ledger.pins.first_key_value().unwrap();
    let identity = *identity;
    let owner = row.owner;
    let record = MaterialRecord::Transfer {
        identity,
        before: owner,
        after: CorrectionMaterialPinOwner::PreparedCommand { key: [8; 32] },
        class: RetentionClass::MandatoryObligation,
    };
    assert_eq!(
        ledger.frame(NonZeroU64::new(ledger.generation + 1).unwrap(), &record),
        Err(Denial::PinOwnerMismatch)
    );
    assert_eq!(ledger.pins[&identity].owner, owner);
}

mod terminal;

#[test]
fn changed_fields_capture_retains_explicit_absence_across_the_material_roundtrip() {
    use super::super::record::{
        RecordedAbsenceKind, RecordedChangedFields, RecordedPreImageAbsence,
    };
    let ledger = catalog();
    let mut record = capture(&ledger);
    let strict = record.encode().unwrap();
    let strict_body: serde_json::Value = serde_json::from_slice(&strict).unwrap();
    assert_eq!(strict_body["version"], 1);
    assert!(!String::from_utf8(strict.clone())
        .unwrap()
        .contains("changed_fields"));
    let MaterialRecord::Capture { material } = &mut record else {
        unreachable!()
    };
    material.changed_fields = Some(RecordedChangedFields {
        role: "kept-record".into(),
        entity: "Record".into(),
        target: entity(),
        absences: vec![RecordedPreImageAbsence {
            entity: "Record".into(),
            aspect: "Optional".into(),
            field: "value".into(),
            target: worth_relational::facade::transactions::RecordRef::Entity(entity()),
            kind: worth_relational::facade::identity::KindId(7),
            location: RecordedLocation {
                aspect: "Optional".into(),
                fields: vec!["value".into()],
            },
            absence: RecordedAbsenceKind::Aspect,
        }],
    });
    let encoded = record.encode().unwrap();
    let mut body: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(body["version"], 2);
    let frame = QueryCorrectionMaterialFrame::prepare(
        ledger.home,
        NonZeroU64::MIN,
        [0; 32],
        NonZeroU64::MIN,
        RetentionClass::MandatoryObligation,
        &encoded,
    )
    .unwrap();
    assert_eq!(MaterialRecord::decode(&frame).unwrap(), record);
    let decoded = MaterialRecord::decode(
        &QueryCorrectionMaterialFrame::readmit(frame.bytes().to_vec()).unwrap(),
    )
    .unwrap();
    assert_eq!(decoded, record);
    body["version"] = serde_json::json!(1);
    let forged_legacy = QueryCorrectionMaterialFrame::prepare(
        ledger.home,
        NonZeroU64::MIN,
        [0; 32],
        NonZeroU64::MIN,
        RetentionClass::MandatoryObligation,
        &serde_json::to_vec(&body).unwrap(),
    )
    .unwrap();
    assert_eq!(
        MaterialRecord::decode(&forged_legacy),
        Err(Denial::MalformedFrame),
        "legacy material cannot acquire authenticated absence by adding a default field"
    );
    if let MaterialRecord::Capture { material } = &mut record {
        material.changed_fields = None;
    }
    assert_eq!(
        record.encode().unwrap(),
        strict,
        "strict producer bytes are unchanged"
    );
}
