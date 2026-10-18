//! Bind checked routing to the exact completed pre-effect carrier.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    application_checkpoint::facts::FactPayload, fact_verification_observation,
};
use std::sync::Arc;

pub(in crate::domain_computation::primary_graph::application_attempt::read_set) struct PreparedComputationFacts
{
    index: Arc<ComputationFactIndex>,
    metadata: Arc<
        worth_execution::ExecutionOrderedMap<WorthQueryApplicationFactKey, SealedComputationFact>,
    >,
    handlers: Option<BTreeMap<WorthQueryApplicationFactKey, usize>>,
    decision_count: usize,
    calls: BTreeSet<ComputationRead>,
}

impl SealedComputationFacts {
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn retain_projection(
        &mut self,
        projection: Arc<super::super::CompletedFactProjection>,
    ) {
        self.charged = self.charged.and_then(|n| n.checked_add(projection.bytes()));
        self.projection = Some(projection);
    }

    pub(in crate::domain_computation::primary_graph) fn original_payload(&self) -> FactPayload {
        self.original.clone()
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn prepare<
        'a,
    >(
        attribution: ComputationFactAttribution,
        ordered: impl Iterator<Item = &'a WorthQueryApplicationFactKey>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<PreparedComputationFacts, super::super::super::retained_decision_facts::StoreDenial>
    {
        let mut readers = attribution.readers;
        let mut handler = attribution.handler.map(|handler| {
            Arc::try_unwrap(handler)
                .unwrap_or_else(|_| panic!("completed expected view released handler custody"))
        });
        let mut handlers = BTreeMap::new();
        let mut metadata = BTreeMap::new();
        let mut decision_count = 0;
        for (ordinal, key) in ordered.enumerate() {
            fact_verification_observation::completion_metadata();
            decision_count += 1;
            if let Some(handler) = &mut handler
                && let Some(key) = handler.take(key)
            {
                handlers.insert(key, ordinal);
            }
            if let Some((key, reads)) = readers.remove_entry(key) {
                metadata.insert(
                    key,
                    SealedComputationFact {
                        ordinal,
                        readers: reads.in_order(),
                    },
                );
            }
        }
        assert!(
            handler.as_ref().is_none_or(BTreeSet::is_empty),
            "seal observed every handler key"
        );
        assert!(readers.is_empty(), "seal observed every owner key");
        let index = Arc::new(ComputationFactIndex::build(&metadata, policy)?);
        let mut retained_metadata = worth_execution::ExecutionOrderedMap::default();
        for (key, value) in &metadata {
            retained_metadata
                .insert_cloned_in(key, value, policy)
                .map_err(super::reattribution::storage_denial)?;
        }
        Ok(PreparedComputationFacts {
            index,
            metadata: Arc::new(retained_metadata),
            handlers: handler.map(|_| handlers),
            decision_count,
            calls: attribution.calls,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn fact(
        &self,
        key: &WorthQueryApplicationFactKey,
    ) -> Option<&WorthQueryApplicationObservedFact> {
        self.original.get(self.metadata.get(key)?.ordinal)
    }

    pub(in crate::domain_computation::primary_graph) fn observation_work_bound(
        &self,
    ) -> Option<u64> {
        self.index.observation_work
    }

    pub(in crate::domain_computation::primary_graph) fn facts(
        &self,
    ) -> impl Iterator<
        Item = (
            &WorthQueryApplicationFactKey,
            &WorthQueryApplicationObservedFact,
            &ComputationFactReaders,
        ),
    > {
        self.metadata.iter().map(|(key, value)| {
            (
                key,
                self.original
                    .get(value.ordinal)
                    .expect("checked original ordinal"),
                &value.readers,
            )
        })
    }

    pub(in crate::domain_computation::primary_graph) fn call_index(
        &self,
    ) -> Arc<ComputationFactIndex> {
        Arc::clone(&self.index)
    }

    pub(in crate::domain_computation::primary_graph) fn at_ordinal(
        &self,
        ordinal: usize,
    ) -> Option<(
        &WorthQueryApplicationFactKey,
        &WorthQueryApplicationObservedFact,
        &ComputationFactReaders,
    )> {
        let key = self.index.ordinals.get(&ordinal)?;
        let value = self.metadata.get(key)?;
        Some((key, self.original.get(ordinal)?, &value.readers))
    }
}

impl PreparedComputationFacts {
    pub(in crate::domain_computation::primary_graph::application_attempt::read_set) fn bind(
        mut self,
        original: FactPayload,
        contracts: Arc<
            worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        >,
        scopes: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        root: worth_relational::facade::identity::EntityId,
        computation_ordinals: Arc<[usize]>,
        content: Option<super::super::super::FactContent>,
    ) -> SealedComputationFacts {
        let layout = CheckedReadLayout::new(
            contracts,
            scopes,
            self.handlers
                .take()
                .expect("actual projection carries handler attribution"),
            root,
            self.decision_count,
            std::mem::take(&mut self.calls),
            computation_ordinals,
        );
        self.finish(
            original,
            layout::SealedReadLayout::Checked(Arc::new(layout)),
            content,
        )
    }

    fn finish(
        self,
        original: FactPayload,
        layout: layout::SealedReadLayout,
        content: Option<super::super::super::FactContent>,
    ) -> SealedComputationFacts {
        let mut result = SealedComputationFacts {
            index: self.index,
            metadata: self.metadata,
            original,
            content,
            charged: None,
            projection: None,
            layout,
        };
        result.charged = result.calculate_charged_bytes();
        result
    }

    #[cfg(test)]
    pub(super) fn bind_fixture(self, original: FactPayload) -> SealedComputationFacts {
        self.finish(original, layout::SealedReadLayout::OwnerFixture, None)
    }
}
