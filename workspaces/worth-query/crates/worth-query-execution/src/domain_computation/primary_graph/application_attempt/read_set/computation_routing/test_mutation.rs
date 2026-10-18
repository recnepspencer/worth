//! Explicit full materialization for adversarial owner fixtures only.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use std::sync::Arc;

impl SealedComputationFacts {
    pub(super) fn at_seal(
        attribution: ComputationFactAttribution,
        observed: &BTreeMap<WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact>,
    ) -> Self {
        Self::prepare(
            attribution,
            observed.keys(),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .bind_fixture(Arc::<[_]>::from(observed.values().cloned().collect::<Vec<_>>()).into())
    }

    pub(in crate::domain_computation::primary_graph) fn replace_fact(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        fact: WorthQueryApplicationObservedFact,
    ) {
        let ordinal = self
            .metadata
            .get(key)
            .expect("an owner call read the key")
            .ordinal;
        let mut values: Vec<_> = self.original.iter().cloned().collect();
        values[ordinal] = fact;
        self.original = Arc::<[_]>::from(values).into();
        self.content = None;
        self.layout = super::layout::SealedReadLayout::OwnerFixture;
        self.charged = self.calculate_charged_bytes();
    }

    pub(in crate::domain_computation::primary_graph) fn copy_key(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        to: WorthQueryApplicationFactKey,
    ) {
        self.move_key(key, to, false);
    }

    pub(in crate::domain_computation::primary_graph) fn rename_key(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        to: WorthQueryApplicationFactKey,
    ) {
        self.move_key(key, to, true);
    }

    fn move_key(
        &mut self,
        key: &WorthQueryApplicationFactKey,
        to: WorthQueryApplicationFactKey,
        remove: bool,
    ) {
        let mut facts: BTreeMap<_, _> = self
            .facts()
            .map(|(key, fact, readers)| (key.clone(), (fact.clone(), readers.clone())))
            .collect();
        let value = facts.get(key).expect("an owner call read the key").clone();
        if remove {
            facts.remove(key);
        }
        facts.insert(to, value);
        let mut values = Vec::new();
        let mut metadata = BTreeMap::new();
        for (key, (fact, readers)) in facts {
            metadata.insert(
                key,
                SealedComputationFact {
                    ordinal: values.len(),
                    readers,
                },
            );
            values.push(fact);
        }
        self.index = Arc::new(
            ComputationFactIndex::build(
                &metadata,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap(),
        );
        let mut retained = worth_execution::ExecutionOrderedMap::default();
        for (key, value) in metadata {
            retained
                .insert_cloned_in(
                    &key,
                    &value,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
        }
        self.metadata = Arc::new(retained);
        self.original = Arc::<[_]>::from(values).into();
        self.content = None;
        self.layout = super::layout::SealedReadLayout::OwnerFixture;
        self.charged = self.calculate_charged_bytes();
    }
}
