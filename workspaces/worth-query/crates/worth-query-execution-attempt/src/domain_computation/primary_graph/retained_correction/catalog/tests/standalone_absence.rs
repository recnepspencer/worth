//! A field capture's standalone optional-field absences survive the material
//! codec, and version 4 frames that cannot carry them refuse by version.
use super::super::super::record::{RecordedAbsenceKind, RecordedPreImageAbsence};
use super::*;

fn absent_due() -> RecordedPreImageAbsence {
    RecordedPreImageAbsence {
        entity: "Task".into(),
        aspect: "TaskFacts".into(),
        field: "due".into(),
        target: worth_relational::facade::transactions::RecordRef::Entity(entity()),
        kind: worth_relational::facade::identity::KindId(7),
        location: RecordedLocation {
            aspect: "TaskFacts".into(),
            fields: vec!["due".into()],
        },
        absence: RecordedAbsenceKind::Field,
    }
}

fn frame(ledger: &MaterialCatalog, body: &[u8]) -> QueryCorrectionMaterialFrame {
    QueryCorrectionMaterialFrame::prepare(
        ledger.home,
        NonZeroU64::MIN,
        [0; 32],
        NonZeroU64::MIN,
        RetentionClass::MandatoryObligation,
        body,
    )
    .unwrap()
}

#[test]
fn a_standalone_absence_roundtrips_and_conflicting_truth_refuses() {
    let ledger = catalog();
    let mut record = capture(&ledger);
    if let MaterialRecord::Capture { material } = &mut record {
        material.absences.push(absent_due());
    }
    let cold = QueryCorrectionMaterialFrame::readmit(
        frame(&ledger, &record.encode().unwrap()).bytes().to_vec(),
    )
    .unwrap();
    assert_eq!(MaterialRecord::decode(&cold).unwrap(), record);
    let mut repeated = record.clone();
    if let MaterialRecord::Capture { material } = &mut repeated {
        material.absences.push(absent_due());
    }
    assert_eq!(repeated.encode(), Err(Denial::MalformedFrame));
    let mut fieldless = record;
    if let MaterialRecord::Capture { material } = &mut fieldless {
        material.preimage = None;
    }
    assert_eq!(fieldless.encode(), Err(Denial::MalformedFrame));
}

#[test]
fn a_version_4_material_frame_is_refused_by_version_never_reinterpreted() {
    let mut ledger = catalog();
    let mut old: serde_json::Value =
        serde_json::from_slice(&capture(&ledger).encode().unwrap()).unwrap();
    old["version"] = 4.into();
    if let Some(material) = old["body"]["material"].as_object_mut() {
        material.remove("absences");
    }
    let frame = frame(&ledger, &serde_json::to_vec(&old).unwrap());
    assert_eq!(
        MaterialRecord::decode(&frame),
        Err(Denial::UnsupportedVersion(4))
    );
    assert_eq!(ledger.fold(&frame), Err(Denial::UnsupportedVersion(4)));
    assert!(ledger.material.is_empty());
}
