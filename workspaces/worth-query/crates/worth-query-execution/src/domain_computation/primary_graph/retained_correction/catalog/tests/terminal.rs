//! Terminal records preserve command authority independently of optional bodies.
use super::*;
#[test]
fn expiry_refuses_a_pinned_original_and_is_terminal_after_release() {
    let mut ledger = available();
    let record = pin(&ledger);
    append(&mut ledger, record);
    let original = original(ledger.home, 1);
    let position = ledger.position;
    assert_eq!(
        ledger.frame(
            NonZeroU64::new(position + 1).unwrap(),
            &MaterialRecord::Expire { original }
        ),
        Err(Denial::MaterialPinned)
    );
    assert_eq!(ledger.position, position);
    let (identity, row) = ledger.pins.first_key_value().unwrap();
    let identity = *identity;
    let owner = row.owner;
    append(
        &mut ledger,
        MaterialRecord::Transfer {
            identity,
            before: owner,
            after: CorrectionMaterialPinOwner::Publication {
                original: Original::from_recorded(original.owner(), 2),
            },
            class: RetentionClass::MandatoryObligation,
        },
    );
    let owner = ledger.pins[&identity].owner;
    let MaterialRecord::Capture { mut material } = capture(&ledger) else {
        unreachable!()
    };
    let inverse = Original::from_recorded(original.owner(), 2);
    material.original = inverse;
    material.key = [6; 32];
    material.input_identity = [10; 32];
    material.intent_identity = [10; 32];
    material.intent = format!("v2:{}:source=-:operation=-:scope=-:precondition=-:input={}:proposal=-:conditional-definition=-:correction={}", "0a".repeat(32), "0a".repeat(32), "07".repeat(32));
    append(&mut ledger, MaterialRecord::Capture { material });
    let MaterialRecord::Ready { mut material } = ready(&ledger) else {
        unreachable!()
    };
    material.original = inverse;
    material.commit.commit_id = CommitId(12);
    append(&mut ledger, MaterialRecord::Ready { material });
    assert_eq!(
        ledger.pin_count(inverse),
        1,
        "publication custody also protects its inverse input until settlement"
    );
    append(
        &mut ledger,
        MaterialRecord::Release {
            identity,
            owner,
            class: RetentionClass::OptionalRetainedMaterial,
        },
    );
    append(&mut ledger, MaterialRecord::Expire { original });
    assert_eq!(
        ledger.material[&original].available().unwrap_err(),
        Denial::RetainedMaterialExpired
    );
    assert_eq!(ledger.material[&original].metadata.key, [4; 32]);
    assert_eq!(
        ledger.material[&original].metadata.intent,
        "recorded exact intent"
    );
    assert!(
        matches!(&ledger.material[&original].body, MaterialBody::Expired { commit } if *commit == receipt())
    );
    assert_eq!(
        ledger.frame(
            NonZeroU64::new(ledger.generation + 1).unwrap(),
            &capture(&ledger)
        ),
        Err(Denial::MaterialAlreadyCaptured)
    );
}

#[test]
fn terminal_command_dispositions_are_authoritative_while_ready_material_is_optional() {
    let ledger = available();
    let original = original(ledger.home, 1);
    let identity =
        CorrectionMaterialPinIdentity::from_recorded(ledger.home, NonZeroU64::new(3).unwrap());
    for record in [
        MaterialRecord::DischargePreparation {
            identity,
            key: [6; 32],
            class: RetentionClass::OptionalRetainedMaterial,
        },
        MaterialRecord::Release {
            identity,
            owner: CorrectionMaterialPinOwner::Publication { original },
            class: RetentionClass::OptionalRetainedMaterial,
        },
        MaterialRecord::Transfer {
            identity,
            before: CorrectionMaterialPinOwner::PreparedCommand { key: [6; 32] },
            after: CorrectionMaterialPinOwner::Publication { original },
            class: RetentionClass::MandatoryObligation,
        },
        MaterialRecord::Abandon { original },
        MaterialRecord::Expire { original },
    ] {
        assert_eq!(record.class(), RetentionClass::Authoritative);
        let frame = QueryCorrectionMaterialFrame::prepare(
            ledger.home,
            NonZeroU64::new(3).unwrap(),
            ledger.digest,
            NonZeroU64::new(3).unwrap(),
            record.class(),
            &record.encode().unwrap(),
        )
        .unwrap();
        assert_eq!(MaterialRecord::decode(&frame).unwrap(), record);
    }
    assert_eq!(
        ready(&ledger).class(),
        RetentionClass::OptionalRetainedMaterial
    );
}

#[test]
fn released_unpublished_owner_discharges_only_its_publication_pin() {
    let mut ledger = available();
    let record = pin(&ledger);
    append(&mut ledger, record);
    let (&identity, row) = ledger.pins.first_key_value().unwrap();
    let before = row.owner;
    let source = original(ledger.home, 1);
    let unpublished = original(ledger.home, 2);
    append(
        &mut ledger,
        MaterialRecord::Transfer {
            identity,
            before,
            after: CorrectionMaterialPinOwner::Publication {
                original: unpublished,
            },
            class: RetentionClass::MandatoryObligation,
        },
    );
    let MaterialRecord::Capture { mut material } = capture(&ledger) else {
        unreachable!()
    };
    material.original = unpublished;
    append(&mut ledger, MaterialRecord::Capture { material });
    assert_eq!(ledger.pin_count(source), 1);
    assert_eq!(ledger.pin_count(unpublished), 1);
    let frame = append(
        &mut ledger,
        MaterialRecord::Abandon {
            original: unpublished,
        },
    );
    assert_eq!(frame.body_retention_class(), RetentionClass::Authoritative);
    assert_eq!(ledger.pin_count(source), 0);
    assert_eq!(ledger.pin_count(unpublished), 0);
    assert!(ledger.material[&source].available().is_ok());
    assert!(matches!(
        ledger.material[&unpublished].body,
        MaterialBody::Abandoned
    ));
    assert_eq!(
        ledger.released[&identity],
        CorrectionMaterialPinOwner::Publication {
            original: unpublished
        }
    );
}
