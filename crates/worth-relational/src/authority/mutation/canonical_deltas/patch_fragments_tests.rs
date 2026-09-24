use worth_foundational::facade::{
    AspectBinding, AspectValue as FoundationalAspectValue, FieldKey,
    InternedString as FoundationalInternedString,
};
use worth_foundational::{aspects, AspectContractRevision, AspectIdentity, ScalarAspectType};

use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::publication::patch::data::{
    PatchDetail, PublishedAuthoritativePatchOperation, PublishedAuthoritativePatchValue,
    RecordStructuralChange,
};
use crate::schema::data::AspectContractPlanRevision;
use crate::transactions::data::RecordRef;

use super::data::{
    CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding,
    LifecycleTransitionClass,
};
use worth_foundational::facade::AspectKey;

#[test]
fn scalar_changed_binding_materializes_foundational_whole_aspect_set() {
    let name_key = AspectKey::new("name").unwrap();
    let fragment = scalar_delta(name_key.clone(), "alice")
        .into_foundational_patch_fragment(PatchDetail::DenseBitset(Vec::new()))
        .expect("foundational scalar patch fragment");

    assert_eq!(fragment.patch.whole_aspect_sets().count(), 1);
    assert_eq!(fragment.patch.whole_aspect_clears().count(), 0);
    let published_record = fragment.published_record();
    assert_eq!(
        published_record.authoritative_changed_aspects(),
        crate::publication::patch::data::ordered_aspect_keys([name_key.clone()])
    );
    assert!(matches!(
        published_record
            .authoritative_patch
            .full_grammar_operations(),
        [PublishedAuthoritativePatchOperation::WholeAspectSet {
            aspect_key,
            value: PublishedAuthoritativePatchValue::Scalar(value),
            ..
        }] if aspect_key == &name_key
            && value == &FoundationalAspectValue::String(
                FoundationalInternedString::from("alice")
            )
    ));
    let semantic = &published_record.semantic_changes[0];
    assert_eq!(
        semantic.kind(),
        worth_foundational::facade::AuthoritativeAspectChangeKind::WholeAspectSet
    );
    assert_eq!(semantic.aspect_identity(), AspectIdentity(9));
    assert_eq!(semantic.contract_revision(), AspectContractRevision(1));
    assert_eq!(semantic.field_path(), None);
}

fn scalar_delta(aspect_key: AspectKey, value: &str) -> CanonicalRecordAspectDelta {
    let target_key = AspectKey::new("target").unwrap();
    let lifecycle_key = AspectKey::new("lifecycle").unwrap();
    CanonicalRecordAspectDelta {
        target: RecordRef::Entity(EntityId::new(PartitionId(1), 0, 1)),
        kind_id: KindId(7),
        plan_revision: AspectContractPlanRevision(1),
        structural_change: RecordStructuralChange::Updated,
        changed_aspects: crate::publication::patch::data::ordered_aspect_keys([aspect_key.clone()]),
        evaluated_bindings: smallvec::smallvec![
            EvaluatedAspectBinding {
                aspect_key: aspect_key.clone(),
                contract: aspects()
                    .contract()
                    .for_key(aspect_key.clone())
                    .identified_by(AspectIdentity(9))
                    .at_revision(AspectContractRevision(1))
                    .scalar(ScalarAspectType::String),
                binding: AspectBinding::EntityField {
                    field: FieldKey::new("name".to_string()).unwrap(),
                },
                changed: true,
                aspect_shape: worth_foundational::AspectShape::Scalar(ScalarAspectType::String),
                evidence: CanonicalAspectDeltaEvidence::ScalarAspectValueTransition {
                    locator: authoritative_value_locator(&aspect_key),
                    old_present: true,
                    new_present: true,
                    old_value: Some(FoundationalAspectValue::String(
                        FoundationalInternedString::from("before"),
                    )),
                    new_value: Some(FoundationalAspectValue::String(
                        FoundationalInternedString::from(value),
                    )),
                },
                field_revision_changes: None,
            },
            EvaluatedAspectBinding {
                aspect_key: target_key.clone(),
                contract: aspects()
                    .contract()
                    .for_key(target_key.clone())
                    .identified_by(AspectIdentity(10))
                    .at_revision(AspectContractRevision(1))
                    .reference_entity(),
                binding: AspectBinding::RelationTargetEndpoint,
                changed: false,
                aspect_shape: worth_foundational::AspectShape::Reference(
                    worth_foundational::ReferenceAspectType::Entity,
                ),
                evidence: CanonicalAspectDeltaEvidence::EndpointIdentity {
                    locator: authoritative_value_locator(&target_key),
                    old: None,
                    new: None,
                },
                field_revision_changes: None,
            },
            EvaluatedAspectBinding {
                aspect_key: lifecycle_key.clone(),
                contract: aspects()
                    .contract()
                    .for_key(lifecycle_key.clone())
                    .identified_by(AspectIdentity(11))
                    .at_revision(AspectContractRevision(1))
                    .scalar(ScalarAspectType::String),
                binding: AspectBinding::LifecycleTransition,
                changed: false,
                aspect_shape: worth_foundational::AspectShape::Scalar(ScalarAspectType::String),
                evidence: CanonicalAspectDeltaEvidence::Lifecycle {
                    locator: authoritative_value_locator(&lifecycle_key),
                    transition: LifecycleTransitionClass::NoTransition,
                },
                field_revision_changes: None,
            }
        ],
        contains_opaque_aspect: false,
    }
}

fn authoritative_value_locator(
    aspect_key: &AspectKey,
) -> worth_foundational::facade::AspectValueLocator {
    worth_foundational::facade::AspectValueLocator::whole_aspect(
        worth_foundational::facade::AspectLocator::new(
            worth_foundational::facade::LocatorAuthority::Authoritative,
            aspect_key.clone(),
        ),
    )
}
