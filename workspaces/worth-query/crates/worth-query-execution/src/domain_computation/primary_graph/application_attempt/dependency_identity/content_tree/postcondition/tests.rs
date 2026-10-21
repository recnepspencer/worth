//! Exercise the production overlay, including new temporary and surviving heaps.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::with_bootstrap_advancement;
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    transactions::planned_single_field_locator,
};

#[test]
fn growing_own_effect_preserves_original_and_publishes_a_distinct_funded_body() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 24), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let request = phase.execution_request();
        let worth_execution::ExecutionAllocationPolicy::Execution(lease) =
            request.allocation_policy()
        else {
            panic!("native payer");
        };
        let control = StorageControl::new(request.allocation_policy(), None);
        let baseline = lease.charged_memory_bytes();
        let entity_id = EntityId::new(PartitionId::main(), 3, 1);
        let kind = KindId::new(7);
        let aspect = AspectKey::new("details").unwrap();
        let locator =
            planned_single_field_locator(aspect.clone(), FieldKey::new("payload").unwrap());
        let before = Fact::Field {
            entity_id,
            kind,
            locator: locator.clone(),
            value: AspectValue::String("tiny".into()),
        };
        let values = vec![
            Fact::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision: Some(9),
            },
            before.clone(),
            before.clone(),
        ];
        // Production completion admits native chunks; opaque original/decoded
        // carriers cannot be relabelled as native patch storage.
        let original_heaps = control.policy().reserve_memory(values.iter().map(ChargedBytes::additional_charged_bytes).sum()).unwrap();
        let mut author = crate::domain_computation::primary_graph::application_checkpoint::facts::NativeFactPayloadBuilder::allocate_completed_read(values.len(), control).unwrap();
        for fact in &values { author.push(fact.clone()).unwrap(); }
        let body = author.seal().unwrap();
        let (original, _) = FactContent::build(body.clone(), control).unwrap();
        let prior = Arc::downgrade(&original.0);
        let effect_value = AspectValue::String("g".repeat(32 * 1024).into());
        let effects = [Effect::UpdateEntity {
            entity: "Entry".into(),
            entity_id,
            fields: BTreeMap::from([(locator.clone(), effect_value.clone())]),
        }];
        let held = lease
            .reserve_memory(lease.available_memory_bytes().expect("this fixture supplies a finite memory ceiling") - 8_192)
            .unwrap();
        assert!(original.postcondition(&effects, &[1, 2], control).is_err());
        assert_eq!(body.iter().cloned().collect::<Vec<_>>(), values);
        assert!(original.same_body(&body));
        drop(held);
        let (normalized, content, moved, _) =
            original.postcondition(&effects, &[1, 2], control).unwrap();
        assert_eq!(moved, [1, 2], "every duplicate read ordinal is preserved");
        assert!(!normalized.same_backing(&body));
        assert!(content.same_body(&normalized));
        let replacement = Fact::Field {
            entity_id,
            kind,
            locator,
            value: effect_value,
        };
        let expected = vec![values[0].clone(), replacement.clone(), replacement];
        assert_eq!(normalized.iter().cloned().collect::<Vec<_>>(), expected);
        assert_eq!(
            body.iter().cloned().collect::<Vec<_>>(),
            values,
            "pre-effect facts stay original"
        );
        assert_eq!(
            content.identity([3; 32]).unwrap().0,
            super::super::oracle::cold(&expected, [3; 32])
        );
        let clone = content.clone();
        drop(content);
        drop(original);
        assert!(
            prior.upgrade().is_none(),
            "a new root retains no history parent"
        );
        drop(normalized);
        assert!(
            lease.charged_memory_bytes() > baseline,
            "the final content clone still holds body and grants"
        );
        drop(clone);
        drop(body);
        drop(original_heaps);
        assert_eq!(lease.charged_memory_bytes(), baseline);
    })
    .unwrap();
}
