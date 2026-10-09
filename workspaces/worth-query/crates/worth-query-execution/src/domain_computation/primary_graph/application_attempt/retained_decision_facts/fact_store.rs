//! Specific fact owner: only local authoring contains RefCell slots. The
//! retained payload is move-only and has no policy/request references.
use super::{AdmittedFactKey, RetainedFact, RetainedFactStore, StorageControl, StoreDenial};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use worth_execution::{ExecutionArray, ExecutionArrayIntoIter};

pub(in crate::domain_computation::primary_graph) struct AuthoringSourceFacts {
    store: RetainedFactStore<Fact>,
    failure: Option<StoreDenial>,
}
pub(in crate::domain_computation::primary_graph) struct RetainedSourceFacts(
    ExecutionArray<RetainedFact<Fact>>,
);
pub(in crate::domain_computation::primary_graph) struct RetainedSourceFactValues(
    ExecutionArrayIntoIter<RetainedFact<Fact>>,
);
impl AuthoringSourceFacts {
    pub(in crate::domain_computation::primary_graph) fn new(
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Ok(Self {
            store: RetainedFactStore::new(control)?,
            failure: None,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn from_retained(
        facts: RetainedSourceFacts,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Ok(Self {
            store: RetainedFactStore::from_sorted_records(facts.0, control)?,
            failure: None,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn vacant() -> Self {
        Self {
            store: RetainedFactStore::vacant(),
            failure: None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn capture(
        &mut self,
        fact: Fact,
        control: StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        if let Some(failure) = &self.failure {
            return Err(failure.clone());
        }
        let result = self.capture_inner(fact, control);
        if let Err(failure) = &result {
            self.failure = Some(failure.clone());
        }
        result
    }
    fn capture_inner(
        &mut self,
        fact: Fact,
        control: StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        let predicate = match &fact {
            Fact::IndexedEntitySelection {
                value,
                candidate_limit,
                ..
            } => Some((value, *candidate_limit)),
            _ => None,
        };
        let key = AdmittedFactKey::from_borrowed(
            |writer| fact.write_dependency_locator(writer),
            predicate,
            control,
        )?;
        self.store
            .insert(key, fact, control, |existing, duplicate| {
                // Storage compared ALL typed-key bytes first; this callback may
                // change only non-key fields. Endpoint union admits its own nested
                // payload while both originals remain retained.
                existing
                    .merge_after_equal_source_key(duplicate, control)?
                    .then_some(())
                    .ok_or(StoreDenial::ConflictingBody)
            })
    }
    /// A shared producer/decision dependency keeps its decision provenance.
    pub(in crate::domain_computation::primary_graph) fn merge_existing(
        &mut self,
        fact: Fact,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<Fact>, StoreDenial> {
        let predicate = match &fact {
            Fact::IndexedEntitySelection {
                value,
                candidate_limit,
                ..
            } => Some((value, *candidate_limit)),
            _ => None,
        };
        let key = AdmittedFactKey::from_borrowed(
            |writer| fact.write_dependency_locator(writer),
            predicate,
            control,
        )?;
        self.store
            .merge_existing(key, fact, control, |existing, duplicate| {
                existing
                    .merge_after_equal_source_key(duplicate, control)?
                    .then_some(())
                    .ok_or(StoreDenial::ConflictingBody)
            })
    }
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        control: StorageControl<'_, '_>,
    ) -> Result<RetainedSourceFacts, StoreDenial> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        Ok(RetainedSourceFacts(self.store.finish(control)?))
    }
}
impl RetainedSourceFacts {
    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.0.len()
    }
    pub(in crate::domain_computation::primary_graph) fn into_values(
        self,
    ) -> RetainedSourceFactValues {
        RetainedSourceFactValues(self.0.into_iter())
    }
}
impl std::fmt::Debug for RetainedSourceFacts {
    fn fmt(&self, writer: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writer
            .debug_struct("RetainedSourceFacts")
            .field("count", &self.len())
            .finish()
    }
}
impl Iterator for RetainedSourceFactValues {
    type Item = Fact;
    fn next(&mut self) -> Option<Fact> {
        self.0.next().map(|record| record.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl ExactSizeIterator for RetainedSourceFactValues {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_actual_fact_payload_is_send_and_sync_without_clone() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<RetainedSourceFacts>();
        send_sync::<crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionSnapshot<()>>();
        send_sync::<crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionSnapshot<(), ()>>();
    }
}
