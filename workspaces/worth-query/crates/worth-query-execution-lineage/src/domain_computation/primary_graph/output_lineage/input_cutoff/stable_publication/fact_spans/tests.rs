//! Span boundaries cannot substitute for exact body or currentness custody.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::facts::NativeFactPayloadBuilder;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_execution::ExecutionAllocationPolicy;
fn fact(ordinal: usize) -> Fact {
    Fact::SourceEntity {
        entity_id: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            (ordinal % 17) as u64,
            1,
        ),
    }
}
fn native(count: usize, control: StorageControl<'_, '_>) -> FactPayload {
    let mut author = NativeFactPayloadBuilder::allocate(count, control).unwrap();
    for ordinal in 0..count {
        author.push(fact(ordinal)).unwrap();
    }
    author.seal().unwrap()
}

#[test]
fn repeated_alias_spans_preserve_prefix_order_duplicates_and_remove_only_witness() {
    let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
    for count in [0, 1, 31, 32, 33, 100] {
        let prefix = native(count, control);
        let witness = native(3, control);
        let suffix = native(2, control);
        let mut body = prefix
            .append_completed(witness, control)
            .unwrap()
            .append_completed(suffix, control)
            .unwrap();
        for _ in 0..20 {
            let spans = StableFactSpans::new(body.clone(), count, 3).unwrap();
            let performed = spans.performed_sequence(&body, count, control).unwrap();
            assert_eq!(performed.len(), count + 2);
            assert!(performed.iter().eq((0..count)
                .chain(0..2)
                .map(fact)
                .collect::<Vec<_>>()
                .iter()));
            for ordinal in 0..count {
                assert!(std::ptr::eq(
                    prefix.get(ordinal).unwrap(),
                    performed.get(ordinal).unwrap()
                ));
            }
            assert!(
                spans
                    .performed_sequence(&native(body.len(), control), count, control)
                    .is_err(),
                "equal data does not qualify different backing"
            );
            assert!(spans.performed_sequence(&body, count + 1, control).is_err());
            body = body
                .slice(0..count)
                .append_completed(native(3, control), control)
                .unwrap()
                .append_completed(native(2, control), control)
                .unwrap();
        }
        assert!(StableFactSpans::new(body.clone(), usize::MAX, 1).is_none());
        assert!(StableFactSpans::new(body.clone(), body.len(), 1).is_none());
    }
}
