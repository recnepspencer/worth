//! Owned graph laws survive the actual schema record and fresh readmission.
use super::{decode_member, encode_member, encode_payload, frame_payload};
use crate::facade::{
    WorthQueryPackageArchiveDecodeWork, WorthQueryPackageArchiveDenialKind as Kind,
    WorthQueryPackageArchiveLimits as Limits, WorthQueryPackageArchiveRecordDecoder,
};
use worth_query_declaration::facade::application_schema::{
    validate_portable_application_schema_freshly as readmit, ApplicationRelationIntegrity,
    ApplicationRelationIntegrityRecord, ApplicationRelationMinimumReachabilityRecord,
    ApplicationRelationUniquenessScope as Scope, ApplicationSchemaDeclarationDenial,
    ApplicationSchemaMember as Member, WorthQueryPortableApplicationSchemaParts as Parts,
    WorthQueryPortableApplicationSchemaRecord as Schema,
};
use worth_query_installation::facade::WorthQueryPortablePackageRecord;

#[test]
fn all_relation_laws_and_owned_kind_names_survive_record_and_schema_identity() {
    for scope in [Scope::DirectedSemanticEdge, Scope::NormalizedSymmetricEdge] {
        let member = relation(scope);
        let bytes = encode_member(&member);
        assert_eq!(
            &bytes[..2],
            &26_u16.to_be_bytes(),
            "enabled laws cannot take the lawless shortcut"
        );
        let decoded = decode_member(&bytes);
        assert_eq!(decoded, member);
        let expected = readmit(schema(member)).unwrap();
        let restored = readmit(schema(decoded.clone())).unwrap();
        assert_eq!(expected.identity(), restored.identity());
        let mut reordered = decoded;
        let Member::Relation { integrity, .. } = &mut reordered else {
            unreachable!()
        };
        integrity.reaches_at_least.as_mut().unwrap().target_kinds =
            vec!["From".to_owned(), target_name()];
        assert_eq!(
            expected.identity(),
            readmit(schema(reordered)).unwrap().identity(),
            "kind-set order and duplicates do not change canonical law meaning"
        );
    }
}

#[test]
fn freshly_decoded_minimum_requires_positive_count_and_completed_kind_membership() {
    for minimum in [
        ApplicationRelationMinimumReachabilityRecord {
            minimum_reachable_targets: 0,
            target_kinds: vec![target_name()],
        },
        ApplicationRelationMinimumReachabilityRecord {
            minimum_reachable_targets: 2,
            target_kinds: vec![],
        },
        ApplicationRelationMinimumReachabilityRecord {
            minimum_reachable_targets: 2,
            target_kinds: vec![format!("Unknown_{}", 42)],
        },
    ] {
        let mut member = relation(Scope::DirectedSemanticEdge);
        let Member::Relation { integrity, .. } = &mut member else {
            unreachable!()
        };
        integrity.reaches_at_least = Some(minimum);
        let decoded = decode_member(&encode_member(&member));
        assert_eq!(
            readmit(schema(decoded)).unwrap_err(),
            ApplicationSchemaDeclarationDenial::InvalidRelationIntegrity
        );
    }
}

#[test]
fn previous_record_grammar_is_refused_before_payload_or_work() {
    let record = schema(relation(Scope::DirectedSemanticEdge));
    let payload = encode_payload(&record, Limits::DEFAULT);
    let mut frame = frame_payload(&payload);
    let mut current = WorthQueryPackageArchiveRecordDecoder::new(Limits::DEFAULT);
    let decoded = current.decode_frame(&frame).unwrap();
    assert_eq!(
        decoded.record(),
        &WorthQueryPortablePackageRecord::ApplicationSchema(record)
    );
    frame[..2].copy_from_slice(&15_u16.to_be_bytes());
    let mut old = WorthQueryPackageArchiveRecordDecoder::new(Limits::DEFAULT);
    assert_eq!(
        old.decode_frame(&frame).unwrap_err().kind(),
        Kind::UnsupportedRecordVersion
    );
    assert_eq!(old.work(), WorthQueryPackageArchiveDecodeWork::default());
}

#[test]
fn orbit_survives_real_records_and_cross_kind_records_refuse_fresh_readmission() {
    let make = |orbit, to: &str| Member::Relation {
        relation: "Successor".into(),
        from: "From".into(),
        to: to.into(),
        integrity: if orbit {
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit()
        } else {
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        }
        .into(),
    };
    let orbit = make(true, "From");
    let decoded = decode_member(&encode_member(&orbit));
    assert_eq!(decoded, orbit);
    let admitted = readmit(schema(decoded)).unwrap();
    assert_eq!(
        admitted.identity(),
        readmit(schema(orbit)).unwrap().identity()
    );
    assert_ne!(
        admitted.identity(),
        readmit(schema(make(false, "From"))).unwrap().identity()
    );
    let hostile = make(true, &target_name());
    assert_eq!(
        readmit(schema(decode_member(&encode_member(&hostile)))).unwrap_err(),
        ApplicationSchemaDeclarationDenial::InvalidRelationIntegrity
    );
}

#[test]
fn minimum_kind_collection_uses_existing_depth_and_entry_admission() {
    let record = schema(relation(Scope::DirectedSemanticEdge));
    let payload = encode_payload(&record, Limits::DEFAULT);
    let frame = frame_payload(&payload);
    let mut depth =
        WorthQueryPackageArchiveRecordDecoder::new(Limits::DEFAULT.with_maximum_nesting_depth(1));
    assert_eq!(
        depth.decode_frame(&frame).unwrap_err().kind(),
        Kind::NestingDepthBudgetExceeded
    );
    assert_eq!(depth.work(), WorthQueryPackageArchiveDecodeWork::default());
    let mut entries = WorthQueryPackageArchiveRecordDecoder::new(
        Limits::DEFAULT.with_maximum_nested_entries(record.members().len() as u64),
    );
    assert_eq!(
        entries.decode_frame(&frame).unwrap_err().kind(),
        Kind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        entries.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

fn relation(scope: Scope) -> Member {
    let mut integrity: ApplicationRelationIntegrityRecord =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .acyclic()
            .symmetric()
            .unique(scope)
            .same_partition_endpoints()
            .into();
    integrity.reaches_at_least = Some(ApplicationRelationMinimumReachabilityRecord {
        minimum_reachable_targets: 2,
        target_kinds: vec![target_name(), "From".to_owned(), target_name()],
    });
    Member::Relation {
        relation: "Edge".to_owned(),
        from: "From".to_owned(),
        to: target_name(),
        integrity,
    }
}

fn target_name() -> String {
    format!("Target_{}", 42)
}

fn schema(relation: Member) -> Schema {
    let mut members = vec![
        Member::Entity {
            entity: "From".to_owned(),
        },
        Member::Entity {
            entity: target_name(),
        },
        relation,
    ];
    members.sort();
    Schema::from_untrusted_parts(Parts {
        owner: "archive.tests".to_owned(),
        name: "RelationLaws".to_owned(),
        major: 1,
        minor: 0,
        members,
        contributions: Vec::new(),
    })
}
