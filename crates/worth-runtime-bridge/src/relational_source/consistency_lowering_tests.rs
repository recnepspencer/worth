//! A change Relational finds inconsistent reaches the Bridge as one
//! `InvalidAuthoritativePatchSemantics` denial carrying Relational's work, and
//! that denial wins over every Bridge gate.

use crate::facade::{
    BridgeAuthoritativePatchLoweringCounters, BridgeRouteErrorKind, TruthSnapshotIdentity,
};
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    AuthoritativeAspectChangeKind, CanonicalFieldPath, FieldKey,
};
use worth_proof::TransitionOutcome;

use crate::relational_source::relational_test_support::{
    exact_change, published_patch, widened_change, WireOperation, WireValue,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::publication::{
    PatchDetail, PatchOrdering, PatchPublicationMode, PatchStreamPosition,
    PublishedAuthoritativeAspectChange, PublishedAuthoritativeFieldSet,
    PublishedAuthoritativePatchEnvelope, PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use worth_relational::facade::transactions::RecordRef;

use super::patch_envelopes::publication_patch_to_bridge_envelope;
use super::RelationalBridgePublicationDenial;

#[test]
fn a_count_mismatch_is_invalid_semantics_with_relational_work() {
    let denial = denied(vec![field_record(Vec::new(), false)]);

    assert_invalid_semantics(
        &denial,
        "semantic change count did not match canonical patch operations",
    );
    assert_eq!(denial.counters(), relational_work(1, 1, 0, 0));
}

#[test]
fn a_claimed_widening_is_invalid_semantics_before_bridge_admission() {
    let denial = denied(vec![field_record(
        vec![widened_change(name_change("name"))],
        false,
    )]);

    assert_invalid_semantics(
        &denial,
        "Relational publication claimed widening before Bridge admission",
    );
    assert_eq!(denial.counters(), relational_work(1, 1, 1, 1));
}

#[test]
fn an_unjustified_change_is_invalid_semantics_naming_the_change() {
    let denial = denied(vec![field_record(vec![name_change("other")], false)]);

    assert_eq!(
        denial.kind(),
        BridgeRouteErrorKind::InvalidAuthoritativePatchSemantics
    );
    assert!(denial
        .to_string()
        .contains("semantic change was not justified by the canonical authoritative patch"));
    assert_eq!(denial.counters(), relational_work(1, 1, 1, 1));
}

#[test]
fn an_opaque_posture_mismatch_is_invalid_semantics() {
    let denial = denied(vec![field_record(vec![name_change("name")], true)]);

    assert_invalid_semantics(
        &denial,
        "opaque aspect posture did not match canonical semantic changes",
    );
    assert_eq!(denial.counters(), relational_work(1, 1, 1, 1));
}

/// A consistent record with an unadmitted opaque change reaches the Bridge,
/// which refuses its precision. The double-fault tests below build on it.
#[test]
fn a_consistent_unadmitted_opaque_record_is_unsupported_precision() {
    let denial = denied(vec![opaque_record(true)]);

    assert_eq!(
        denial.kind(),
        BridgeRouteErrorKind::UnsupportedAuthoritativePatchPrecision
    );
}

/// An unadmitted opaque change alone is the Bridge's precision refusal.
/// Once the same record is also inconsistent, Relational's denial wins.
#[test]
fn an_inconsistent_opaque_record_is_invalid_semantics_not_unsupported_precision() {
    let denial = denied(vec![opaque_record(false)]);

    assert_invalid_semantics(
        &denial,
        "opaque aspect posture did not match canonical semantic changes",
    );
    assert_eq!(denial.counters(), relational_work(1, 1, 1, 1));
}

/// Relational checks every record before the Bridge gates any, so a later
/// inconsistent record outranks an earlier unadmitted opaque one.
#[test]
fn a_later_inconsistent_record_outranks_an_earlier_unadmitted_opaque_record() {
    let denial = denied(vec![opaque_record(true), field_record(Vec::new(), false)]);

    assert_invalid_semantics(
        &denial,
        "semantic change count did not match canonical patch operations",
    );
    assert_eq!(denial.counters(), relational_work(2, 2, 1, 1));
}

fn denied(records: Vec<PublishedAuthoritativeRecordPatch>) -> RelationalBridgePublicationDenial {
    let patch = PublishedAuthoritativePatchEnvelope {
        ordering: PatchOrdering::CanonicalCommitOrder,
        publication_mode: PatchPublicationMode::CommitNative,
        position: PatchStreamPosition(11),
        authoritative_record_patches: records,
    };
    match publication_patch_to_bridge_envelope(
        CommitId(11),
        &BranchId("main".to_owned()),
        TruthSnapshotIdentity::from_relational_snapshot(
            crate::facade::RelationalBridgeSnapshotIdentityParts::new(11, 1),
        ),
        &patch,
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("an inconsistent publication must be denied"),
    }
}

fn assert_invalid_semantics(denial: &RelationalBridgePublicationDenial, detail: &str) {
    assert_eq!(
        denial.kind(),
        BridgeRouteErrorKind::InvalidAuthoritativePatchSemantics
    );
    assert!(
        denial.to_string().contains(detail),
        "`{denial}` should carry `{detail}`"
    );
}

/// Relational's consistency work, one operation and one expected change per
/// record, as the Bridge lowering counters carry it.
fn relational_work(
    records: u64,
    operations: u64,
    examined: u64,
    matched: u64,
) -> BridgeAuthoritativePatchLoweringCounters {
    BridgeAuthoritativePatchLoweringCounters {
        record_patches_inspected: records,
        authoritative_operations_inspected: operations,
        expected_operations_materialized: operations,
        semantic_changes_inspected: examined,
        semantic_changes_matched: matched,
        ..BridgeAuthoritativePatchLoweringCounters::default()
    }
}

fn profile_binding() -> AspectBinding {
    AspectBinding::EntityField {
        field: FieldKey::new("profile").unwrap(),
    }
}

/// A field set on `profile.<field>`; only `name` matches [`field_record`].
fn name_change(field: &str) -> PublishedAuthoritativeAspectChange {
    exact_change(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(41),
        AspectContractRevision(7),
        profile_binding(),
        AuthoritativeAspectChangeKind::FieldSet,
        Some(CanonicalFieldPath::single(FieldKey::new(field).unwrap())),
    )
}

/// One field patch setting `profile.name`.
fn field_record(
    semantic_changes: Vec<PublishedAuthoritativeAspectChange>,
    contains_opaque_aspect: bool,
) -> PublishedAuthoritativeRecordPatch {
    PublishedAuthoritativeRecordPatch {
        target: RecordRef::Entity(EntityId::new(PartitionId::main(), 11, 1)),
        structural_change: RecordStructuralChange::Updated,
        authoritative_patch: published_patch(vec![WireOperation::FieldLevelPatch {
            aspect_key: AspectKey::new("profile").unwrap(),
            aspect_identity: AspectIdentity(41),
            contract_revision: AspectContractRevision(7),
            binding: profile_binding(),
            field_sets: vec![PublishedAuthoritativeFieldSet {
                field: FieldKey::new("name").unwrap(),
                value: AspectValue::String("after".into()),
            }],
            field_clears: Vec::new(),
        }]),
        semantic_changes,
        contains_opaque_aspect,
        detail: PatchDetail::DenseBitset(Vec::new()),
    }
}

/// One whole-aspect write described as an opaque change, consistent only
/// when the record admits its opaque aspect.
fn opaque_record(contains_opaque_aspect: bool) -> PublishedAuthoritativeRecordPatch {
    let key = AspectKey::new("opaque-payload").unwrap();
    let binding = AspectBinding::EntityField {
        field: FieldKey::new("payload").unwrap(),
    };
    PublishedAuthoritativeRecordPatch {
        target: RecordRef::Entity(EntityId::new(PartitionId::main(), 12, 1)),
        structural_change: RecordStructuralChange::Updated,
        authoritative_patch: published_patch(vec![WireOperation::WholeAspectSet {
            aspect_key: key.clone(),
            aspect_identity: AspectIdentity(51),
            contract_revision: AspectContractRevision(4),
            binding: binding.clone(),
            value: WireValue::Scalar(AspectValue::String("opaque".into())),
        }]),
        semantic_changes: vec![exact_change(
            key,
            AspectIdentity(51),
            AspectContractRevision(4),
            binding,
            AuthoritativeAspectChangeKind::Opaque,
            None,
        )],
        contains_opaque_aspect,
        detail: PatchDetail::DenseBitset(Vec::new()),
    }
}
