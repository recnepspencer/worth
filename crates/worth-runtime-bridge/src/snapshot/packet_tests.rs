use worth_foundational::facade::{
    aspects, AspectContract, AspectContractRevision, AspectFieldLocator, AspectIdentity, AspectKey,
    AspectLocator, AspectMask, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
    ProjectionMask, ScalarAspectType,
};

use crate::mapping::SubscriptionSliceKind;
use crate::snapshot::{
    validate_snapshot_read_result_contract, BridgeSnapshotReadErrorKind, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadRequest,
};

#[test]
fn native_record_read_cannot_be_correlated_by_its_reporting_label() {
    let record = crate::facade::RelationalBridgeRecordIdentityParts::entity(3, 7, 2);
    let contract = SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String);
    let native = SnapshotReadRequest::for_relational_record(record, contract.clone());
    let external =
        SnapshotReadRequest::for_coarse(record.terminal_projection_for_reporting(), contract);

    assert_eq!(native.entity_identity(), external.entity_identity());
    assert_ne!(native.correlation_id(), external.correlation_id());
    assert_eq!(native.relational_record_identity_parts(), Some(record));
    assert_eq!(external.relational_record_identity_parts(), None);
}

#[test]
fn packet_preserves_declared_read_order() {
    let packet = SnapshotReadPacket::new(vec![
        SnapshotReadRequest::for_coarse(
            "user-1",
            SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        ),
        native_subscription_read(
            "user-2",
            SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
            Some(field_path("name")),
            SubscriptionSliceKind::SignalField,
        ),
    ]);

    assert_ne!(
        packet.reads()[0].correlation_id(),
        packet.reads()[1].correlation_id()
    );
    assert_eq!(packet.reads()[0].entity_identity(), "user-1");
    assert_eq!(packet.reads()[1].entity_identity(), "user-2");
    assert_eq!(
        packet.reads()[0].native_target_basis(),
        "snapshot-read-target|locator=version=bridge.snapshot-read-target.v1;domain=locator;entries=[locus=named:aspect.aspect_key,kind=locator,value=exact-text:profile;locus=named:aspect.authority,kind=locator,value=exact-text:planned;locus=named:aspect.kind,kind=locator,value=exact-text:aspect]|projection-mask=version=bridge.snapshot-read-target.v1;domain=aspect-mask;entries=[locus=named:profile.projection.whole,kind=mask,value=exact-text:whole]"
    );
    assert_eq!(
        packet.reads()[1].native_target_basis(),
        "snapshot-read-target|locator=version=bridge.snapshot-read-target.v1;domain=locator;entries=[locus=named:aspect_field.aspect_key,kind=locator,value=exact-text:profile;locus=named:aspect_field.authority,kind=locator,value=exact-text:planned;locus=named:aspect_field.field_path,kind=locator,value=exact-text:name;locus=named:aspect_field.kind,kind=locator,value=exact-text:aspect]|projection-mask=version=bridge.snapshot-read-target.v1;domain=aspect-mask;entries=[locus=named:profile.projection.field.name,kind=mask,value=exact-text:name]"
    );
    assert_eq!(packet.reads()[0].slice_kind(), None);
    assert!(packet.reads()[1].target().field_locator().is_some());
    assert_eq!(
        packet.reads()[1].slice_kind(),
        Some(&SubscriptionSliceKind::SignalField)
    );
    assert!(packet.digest().starts_with("snapshot-read-packet:sha256:"));
}

#[test]
fn packet_request_carries_foundational_projection_target() {
    let packet = SnapshotReadPacket::new(vec![native_subscription_read(
        "user-2",
        profile_struct_read_contract(),
        Some(field_path("name")),
        SubscriptionSliceKind::SignalField,
    )]);
    let target = packet.reads()[0].target();

    assert_eq!(
        target.aspect_locator().authority(),
        LocatorAuthority::Planned
    );
    assert_eq!(target.aspect_key().as_str(), "profile");
    assert_eq!(
        target
            .field_locator()
            .expect("field slice should carry a foundational field locator")
            .field_path()
            .fields()[0]
            .as_str(),
        "name"
    );
    assert_eq!(target.projection_mask().paths().len(), 1);
}

#[test]
fn request_canonical_basis_consumes_target_identity_not_native_target_basis() {
    let read = native_subscription_read(
        "user-2",
        profile_struct_read_contract(),
        Some(field_path("name")),
        SubscriptionSliceKind::SignalField,
    );
    let request_basis = read.canonical_basis();

    assert!(
        request_basis.contains(read.target().target_identity().as_str()),
        "snapshot request basis must consume the typed target proof: {request_basis}"
    );
    assert!(
        !request_basis.contains(read.native_target_basis()),
        "snapshot request basis must not embed native target basis: {request_basis}"
    );
    assert!(read
        .target()
        .target_identity()
        .as_str()
        .starts_with("snapshot-read-target:sha256:"));
    assert!(!read
        .target()
        .target_identity()
        .as_str()
        .contains("snapshot-read-target|locator="));
}

#[test]
fn packet_request_keeps_non_field_slice_mask_whole_aspect() {
    let packet = SnapshotReadPacket::new(vec![native_subscription_read(
        "user-2",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        None,
        SubscriptionSliceKind::SignalRegion,
    )]);
    let target = packet.reads()[0].target();

    assert!(target.field_locator().is_none());
    assert!(target.projection_mask().is_whole_aspect());
}

#[test]
fn validation_preserves_authoritative_absence_as_a_typed_posture() {
    let read = SnapshotReadRequest::for_coarse(
        "user-missing",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    );
    let packet = SnapshotReadPacket::new(vec![read]);
    let result = SnapshotReadPacketResult::new(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![SnapshotReadRecord::absent_for_request(&packet.reads()[0])],
    );

    let validated = validate_snapshot_read_result_contract(&packet, result).unwrap();
    assert!(validated.records()[0].is_absent());
    assert!(validated.records()[0].validated_value_posture().is_none());
}

#[test]
fn validation_rejects_unknown_struct_field_projection_mask() {
    let read = native_subscription_read(
        "user-2",
        profile_struct_read_contract(),
        Some(field_path("missing")),
        SubscriptionSliceKind::SignalField,
    );
    let packet = SnapshotReadPacket::new(vec![read.clone()]);

    let error = validate_snapshot_read_result_contract(
        &packet,
        SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            vec![SnapshotReadRecord::for_request(
                &read,
                worth_foundational::facade::AspectValue::String(("alice").into()),
            )],
        ),
    )
    .expect_err("unknown struct projection field must fail before materialization");

    assert_eq!(
        error.kind(),
        BridgeSnapshotReadErrorKind::ProjectionMaskRejected
    );
    assert!(error.mask_denial().is_some());
}

#[test]
fn packet_digest_changes_when_declared_reads_change() {
    let left = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        "user-1",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    )]);
    let right = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        "user-2",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    )]);

    assert_ne!(left.digest(), right.digest());
}

#[test]
fn packet_result_retains_snapshot_identity() {
    let read = SnapshotReadRequest::for_coarse(
        "a",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    );
    let result = SnapshotReadPacketResult::new(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![SnapshotReadRecord::for_request(
            &read,
            worth_foundational::facade::AspectValue::String(("alice").into()),
        )],
    );

    assert_eq!(
        result.snapshot_identity().canonical_label(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").canonical_label()
    );
    assert_eq!(result.records()[0].correlation_id(), read.correlation_id());
}

#[test]
fn validation_rejects_missing_required_record() {
    let packet = SnapshotReadPacket::new(vec![
        SnapshotReadRequest::for_coarse(
            "user-1",
            SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        ),
        SnapshotReadRequest::for_coarse(
            "user-2",
            SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        ),
    ]);
    let unrelated_read = SnapshotReadRequest::for_coarse(
        "unrelated",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    );

    let error = validate_snapshot_read_result_contract(
        &packet,
        SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            vec![SnapshotReadRecord::for_request(
                &unrelated_read,
                worth_foundational::facade::AspectValue::String(("alice").into()),
            )],
        ),
    )
    .expect_err("missing records must fail the bridge snapshot contract");

    assert_eq!(
        error.kind(),
        BridgeSnapshotReadErrorKind::RecordCountMismatch
    );
    assert_eq!(error.correlation_id(), None);
}

#[test]
fn validation_rejects_duplicate_result_keys() {
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        "user-1",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    )]);
    let duplicate_read = packet.reads()[0].clone();

    let error = validate_snapshot_read_result_contract(
        &packet,
        SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            vec![
                SnapshotReadRecord::for_request(
                    &duplicate_read,
                    worth_foundational::facade::AspectValue::String(("alice").into()),
                ),
                SnapshotReadRecord::for_request(
                    &duplicate_read,
                    worth_foundational::facade::AspectValue::String(("alice-2").into()),
                ),
            ],
        ),
    )
    .expect_err("duplicate result keys must fail the bridge snapshot contract");

    assert_eq!(error.kind(), BridgeSnapshotReadErrorKind::DuplicateRecord);
    assert_eq!(
        error.correlation_id(),
        Some(duplicate_read.correlation_id())
    );
}

#[test]
fn validation_rejects_value_family_outside_read_contract() {
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        "user-1",
        SnapshotReadContract::scalar(aspect_key("profile.score"), ScalarAspectType::Int64),
    )]);
    let read = packet.reads()[0].clone();

    let error = validate_snapshot_read_result_contract(
        &packet,
        SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            vec![SnapshotReadRecord::for_request(
                &read,
                AspectValue::String("not-an-int".into()),
            )],
        ),
    )
    .expect_err("snapshot value family mismatch must fail before materialization");

    assert_eq!(
        error.kind(),
        BridgeSnapshotReadErrorKind::AspectContractValidationDenied
    );
    assert!(error.validation_denial().is_some());
    assert_eq!(
        error.aspect_key().map(AspectKey::as_str),
        Some("profile.score")
    );
}

#[test]
fn validation_accepts_foundational_struct_read_value() {
    let packet = SnapshotReadPacket::new(vec![SnapshotReadRequest::for_coarse(
        "user-1",
        profile_struct_read_contract(),
    )]);
    let read = packet.reads()[0].clone();
    let struct_value = aspects()
        .vocabulary()
        .struct_value()
        .with_field("name", AspectValue::String("alice".into()))
        .finish()
        .expect("valid test struct value");

    let validated = validate_snapshot_read_result_contract(
        &packet,
        SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            vec![SnapshotReadRecord::for_request(&read, struct_value)],
        ),
    )
    .expect("struct snapshot value should validate against struct aspect contract");

    assert!(validated.records()[0].scalar_aspect_value().is_none());
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid test aspect key")
}

fn profile_struct_read_contract() -> SnapshotReadContract {
    let shape = aspects()
        .struct_fields()
        .required("name", ScalarAspectType::String)
        .finish()
        .expect("valid test struct aspect shape");
    SnapshotReadContract::new(AspectContract::struct_aspect(
        aspect_key("profile"),
        AspectIdentity(41),
        AspectContractRevision(1),
        shape,
    ))
}

fn native_subscription_read(
    entity_identity: &str,
    contract: SnapshotReadContract,
    field_path: Option<CanonicalFieldPath>,
    slice_kind: SubscriptionSliceKind,
) -> SnapshotReadRequest {
    let aspect_locator =
        AspectLocator::new(LocatorAuthority::Planned, contract.aspect_key().clone());
    let field_locator = field_path
        .clone()
        .map(|path| AspectFieldLocator::from_aspect(aspect_locator.clone(), path));
    let projection_mask = field_path
        .map(|path| AspectMask::<ProjectionMask>::new([path]))
        .unwrap_or_else(AspectMask::whole_aspect);
    SnapshotReadRequest::for_native_subscription_slice(
        entity_identity,
        contract,
        aspect_locator,
        field_locator,
        projection_mask,
        slice_kind,
    )
}

fn field_path(field_name: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::single(FieldKey::new(field_name.to_owned()).expect("valid field key"))
}
