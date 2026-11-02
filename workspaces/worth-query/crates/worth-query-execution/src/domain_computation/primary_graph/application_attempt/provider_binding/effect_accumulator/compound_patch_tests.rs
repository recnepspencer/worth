//! Composition preserves record identity, every patch, and its independent evidence.
use super::*;
use crate::domain_computation::WorthQueryProvisionalEffectAction;
use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, AspectKey, AspectValue, ContractValidationInput,
    FieldKey, PortableAspectContractBasis, PortableAspectFieldSet, PortableAspectPatchOperation,
};
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    transactions::EntityMutationIntent,
};

fn part(entity_id: EntityId, name: &str, retire: bool) -> WorthQueryLoweredProviderEffect {
    let action = if retire {
        WorthQueryProvisionalEffectAction::Retire {
            target_identity: name.into(),
        }
    } else {
        WorthQueryProvisionalEffectAction::Replace {
            target_identity: name.into(),
        }
    };
    WorthQueryLoweredProviderEffect::EntityAspectPatch {
        entity_id,
        operations: vec![{
            let basis = PortableAspectContractBasis::new(
                AspectKey::new(name).unwrap(),
                AspectIdentity(1),
                AspectContractRevision(1),
            );
            if retire {
                PortableAspectPatchOperation::ClearWhole { basis }
            } else if name.starts_with("new-") {
                PortableAspectPatchOperation::SetWhole {
                    basis,
                    value: ContractValidationInput::Scalar(AspectValue::UInt64(19)),
                }
            } else {
                PortableAspectPatchOperation::PatchFields {
                    basis,
                    selected_fields: vec![FieldKey::new("value").unwrap()],
                    field_sets: vec![PortableAspectFieldSet::new(
                        FieldKey::new("value").unwrap(),
                        AspectValue::UInt64(23),
                    )],
                    field_clears: Vec::new(),
                }
            }
        }],
        steps: vec![WorthQueryProvisionalEffectStep::new("mutation", action).unwrap()],
    }
}

#[test]
fn disjoint_parts_move_to_one_intent_per_full_entity_identity() {
    // Literal identity and operation-roster oracle; no native validity is claimed here.
    for count in [1, 8, 64] {
        let ids: Vec<_> = (0..count)
            .flat_map(|n| {
                [
                    EntityId::new(PartitionId::main(), n + 1, 1),
                    EntityId::new(PartitionId::main(), n + 1, 2),
                    EntityId::new(PartitionId::new(7), n + 1, 1),
                ]
            })
            .collect();
        let effects = ids
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(n, id)| {
                [
                    part(id, &format!("old-{n}"), true),
                    part(id, &format!("new-{n}"), false),
                    part(id, &format!("class-{n}"), false),
                ]
            })
            .collect();
        let (intents, emissions, evidence) = materialize_commit_projections(effects).unwrap();
        assert_eq!(intents.len(), ids.len());
        assert!(emissions.is_empty());
        assert_eq!(evidence.steps().len(), 3 * ids.len());
        assert_eq!(evidence.preparation_work().key_lookups(), 3 * ids.len());
        let mut seen = Vec::new();
        for intent in intents {
            let MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch)) = intent
            else {
                panic!("one native patch");
            };
            let n = ids
                .iter()
                .position(|id| *id == patch.entity_id)
                .expect("literal identity roster");
            assert!(!seen.contains(&n), "each full identity occurs exactly once");
            seen.push(n);
            assert_eq!(
                patch
                    .aspect_patch
                    .operations()
                    .iter()
                    .map(|op| op.basis().key().as_str())
                    .collect::<Vec<_>>(),
                [format!("old-{n}"), format!("new-{n}"), format!("class-{n}")]
            );
            assert!(matches!(
                patch.aspect_patch.operations(),
                [
                    PortableAspectPatchOperation::ClearWhole { .. },
                    PortableAspectPatchOperation::SetWhole {
                        value: ContractValidationInput::Scalar(AspectValue::UInt64(19)),
                        ..
                    },
                    PortableAspectPatchOperation::PatchFields { .. }
                ]
            ));
            let PortableAspectPatchOperation::PatchFields {
                selected_fields,
                field_sets,
                field_clears,
                ..
            } = &patch.aspect_patch.operations()[2]
            else {
                panic!("field patch retained");
            };
            assert_eq!(selected_fields, &[FieldKey::new("value").unwrap()]);
            assert_eq!(
                field_sets,
                &[PortableAspectFieldSet::new(
                    FieldKey::new("value").unwrap(),
                    AspectValue::UInt64(23)
                )]
            );
            assert!(field_clears.is_empty());
        }
    }
}

#[test]
fn corrupted_composed_evidence_is_still_checked_by_the_step_owner() {
    let id = EntityId::new(PartitionId::main(), 1, 1);
    // Mutate the second effect's action on the same fact: the independently
    // owned expected-step checker must refuse, rather than losing either part.
    let denial =
        materialize_commit_projections(vec![part(id, "same", true), part(id, "same", false)])
            .err()
            .expect("conflicting evidence refuses");
    assert_eq!(
        denial.kind(),
        super::super::super::WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep
    );
    assert_eq!(denial.subject(), "same");
}
