use serde::Serialize;
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    AuthoritativeAspectChangeKind, CanonicalFieldPath, FieldKey,
};

use super::consistency::check_change_consistency;
use super::RelationalChangeConsistencyDenialKind as Kind;
use crate::identity::data::{EntityId, PartitionId};
use crate::publication::patch::data::{
    PatchDetail, PatchOrdering, PatchPublicationMode, PatchStreamPosition,
    PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatch,
    PublishedAuthoritativePatchEnvelope, PublishedAuthoritativePatchOperation,
    PublishedAuthoritativePatchValue, PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use crate::transactions::data::RecordRef;

#[test]
fn a_consistent_change_reports_the_work_it_checked() {
    let work = check_change_consistency(&field_patch_with(vec![exact_name_change()], false))
        .expect("the canonical semantic change is consistent");

    assert_eq!(work.records_checked(), 1);
    assert_eq!(work.operations_checked(), 1);
    assert_eq!(work.expected_changes_materialized(), 1);
    assert_eq!(work.semantic_changes_examined(), 1);
    assert_eq!(work.semantic_changes_matched(), 1);
}

#[test]
fn copied_semantic_metadata_cannot_override_the_canonical_patch() {
    let field_binding = || AspectBinding::EntityField {
        field: FieldKey::new("profile".to_string()).unwrap(),
    };
    let cases = [
        semantic_case(
            AspectIdentity(99),
            AspectContractRevision(7),
            "name",
            AuthoritativeAspectChangeKind::FieldSet,
            field_binding(),
        ),
        semantic_case(
            AspectIdentity(41),
            AspectContractRevision(99),
            "name",
            AuthoritativeAspectChangeKind::FieldSet,
            field_binding(),
        ),
        semantic_case(
            AspectIdentity(41),
            AspectContractRevision(7),
            "other",
            AuthoritativeAspectChangeKind::FieldSet,
            field_binding(),
        ),
        semantic_case(
            AspectIdentity(41),
            AspectContractRevision(7),
            "name",
            AuthoritativeAspectChangeKind::FieldClear,
            field_binding(),
        ),
        semantic_case(
            AspectIdentity(41),
            AspectContractRevision(7),
            "name",
            AuthoritativeAspectChangeKind::FieldSet,
            AspectBinding::RelationSourceEndpoint,
        ),
    ];

    for semantic_change in cases {
        let denial = check_change_consistency(&field_patch_with(vec![semantic_change], false))
            .expect_err("drifted semantic metadata must be refused");
        assert_eq!(denial.kind(), Kind::UnjustifiedSemanticChange);
        assert_eq!(denial.work().semantic_changes_matched(), 1);
    }
}

#[test]
fn a_missing_semantic_change_is_a_count_mismatch() {
    let denial = check_change_consistency(&field_patch_with(Vec::new(), false))
        .expect_err("an operation without its semantic change must be refused");

    assert_eq!(denial.kind(), Kind::SemanticChangeCountMismatch);
    assert_eq!(
        denial.detail(),
        "semantic change count did not match canonical patch operations"
    );
    assert_eq!(denial.work().expected_changes_materialized(), 1);
    assert_eq!(denial.work().semantic_changes_examined(), 0);
}

#[test]
fn a_published_change_cannot_claim_widened_precision() {
    let denial = check_change_consistency(&field_patch_with(vec![widened_name_change()], false))
        .expect_err("widening belongs to the consumer, not the publication");

    assert_eq!(denial.kind(), Kind::WidenedPrecisionClaimed);
}

#[test]
fn the_opaque_flag_must_match_the_semantic_changes() {
    let denial = check_change_consistency(&field_patch_with(vec![exact_name_change()], true))
        .expect_err("a record claiming an opaque aspect it lacks must be refused");

    assert_eq!(denial.kind(), Kind::OpaquePostureMismatch);
    assert_eq!(denial.work().semantic_changes_matched(), 1);
}

#[test]
fn a_derived_endpoint_label_cannot_reclassify_an_entity_field_operation() {
    let key = AspectKey::new("profile").unwrap();
    let mut patch = field_patch_with(Vec::new(), false);
    let record = &mut patch.authoritative_record_patches[0];
    record.authoritative_patch = PublishedAuthoritativePatch::new(vec![
        PublishedAuthoritativePatchOperation::WholeAspectSet {
            aspect_key: key.clone(),
            aspect_identity: AspectIdentity(61),
            contract_revision: AspectContractRevision(2),
            binding: AspectBinding::EntityField {
                field: FieldKey::new("profile").unwrap(),
            },
            value: PublishedAuthoritativePatchValue::Scalar(AspectValue::String("after".into())),
        },
    ]);
    record.semantic_changes = vec![PublishedAuthoritativeAspectChange::exact(
        key,
        AspectIdentity(61),
        AspectContractRevision(2),
        AspectBinding::RelationSourceEndpoint,
        AuthoritativeAspectChangeKind::RelationSourceEndpoint,
        None,
    )];

    let denial = check_change_consistency(&patch)
        .expect_err("an endpoint label cannot describe an entity field write");

    assert_eq!(denial.kind(), Kind::UnjustifiedSemanticChange);
}

fn exact_name_change() -> PublishedAuthoritativeAspectChange {
    semantic_case(
        AspectIdentity(41),
        AspectContractRevision(7),
        "name",
        AuthoritativeAspectChangeKind::FieldSet,
        AspectBinding::EntityField {
            field: FieldKey::new("profile".to_string()).unwrap(),
        },
    )
}

/// The exact name change, carrying a widened precision no Relational
/// constructor issues. Only a decoded publication can carry one.
fn widened_name_change() -> PublishedAuthoritativeAspectChange {
    #[derive(Serialize)]
    struct WireChange {
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        kind: AuthoritativeAspectChangeKind,
        field_path: Option<CanonicalFieldPath>,
        precision: PublishedAspectChangePrecision,
    }
    let exact = exact_name_change();
    let wire = WireChange {
        aspect_key: exact.aspect_key().clone(),
        aspect_identity: exact.aspect_identity(),
        contract_revision: exact.contract_revision(),
        binding: exact.binding().clone(),
        kind: exact.kind(),
        field_path: exact.field_path().cloned(),
        precision: PublishedAspectChangePrecision::DeclaredWidening,
    };
    let bytes = rmp_serde::to_vec(&wire).expect("encode the wire change");
    let widened: PublishedAuthoritativeAspectChange =
        rmp_serde::from_slice(&bytes).expect("decode the wire change");
    assert_eq!(
        widened.precision(),
        PublishedAspectChangePrecision::DeclaredWidening
    );
    widened
}

fn semantic_case(
    identity: AspectIdentity,
    revision: AspectContractRevision,
    path: &str,
    kind: AuthoritativeAspectChangeKind,
    binding: AspectBinding,
) -> PublishedAuthoritativeAspectChange {
    PublishedAuthoritativeAspectChange::exact(
        AspectKey::new("profile").unwrap(),
        identity,
        revision,
        binding,
        kind,
        Some(CanonicalFieldPath::single(
            FieldKey::new(path.to_string()).unwrap(),
        )),
    )
}

fn field_patch_with(
    semantic_changes: Vec<PublishedAuthoritativeAspectChange>,
    contains_opaque_aspect: bool,
) -> PublishedAuthoritativePatchEnvelope {
    PublishedAuthoritativePatchEnvelope {
        ordering: PatchOrdering::CanonicalCommitOrder,
        publication_mode: PatchPublicationMode::CommitNative,
        position: PatchStreamPosition(5),
        authoritative_record_patches: vec![PublishedAuthoritativeRecordPatch {
            target: RecordRef::Entity(EntityId::new(PartitionId::main(), 11, 1)),
            structural_change: RecordStructuralChange::Updated,
            authoritative_patch: PublishedAuthoritativePatch::new(vec![
                PublishedAuthoritativePatchOperation::FieldLevelPatch {
                    aspect_key: AspectKey::new("profile").unwrap(),
                    aspect_identity: AspectIdentity(41),
                    contract_revision: AspectContractRevision(7),
                    binding: AspectBinding::EntityField {
                        field: FieldKey::new("profile").unwrap(),
                    },
                    field_sets: vec![PublishedAuthoritativeFieldSet {
                        field: FieldKey::new("name".to_string()).unwrap(),
                        value: AspectValue::String("after".into()),
                    }],
                    field_clears: Vec::new(),
                },
            ]),
            semantic_changes,
            contains_opaque_aspect,
            detail: PatchDetail::DenseBitset(Vec::new()),
        }],
    }
}
