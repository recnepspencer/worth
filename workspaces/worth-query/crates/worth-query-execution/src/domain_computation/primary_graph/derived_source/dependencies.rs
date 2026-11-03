//! Canonical funded dependency batches preserve actual row acquisition provenance.
use super::*;
use crate::domain_computation::primary_graph::structural_walk::memory;
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
#[derive(Debug)]
pub struct DerivedSourceDependencies {
    values: Vec<Arc<DerivedSourceObservation>>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl DerivedSourceDependencies {
    pub(in crate::domain_computation::primary_graph) fn values(
        &self,
    ) -> &[Arc<DerivedSourceObservation>] {
        &self.values
    }
    pub(in crate::domain_computation::primary_graph) fn append(
        prior: Option<&Arc<Self>>,
        body: &Arc<DerivedSourceObservation>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<Arc<Self>, Cause> {
        Self::extend(
            prior,
            std::slice::from_ref(body),
            execution,
            context,
            control,
        )
    }
    /// Capture-set consumers merge one whole step, rather than cloning a growing
    /// vector for every frontier anchor. Row consumers append only their own body.
    pub(in crate::domain_computation::primary_graph) fn extend(
        prior: Option<&Arc<Self>>,
        bodies: &[Arc<DerivedSourceObservation>],
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<Arc<Self>, Cause> {
        let original = prior.map_or(&[][..], |p| p.values());
        let mut tree_memory = None;
        let mut tree_bytes = 0;
        #[expect(
            clippy::mutable_key_type,
            reason = "DerivedSourceKey orders only immutable relation, identity, root and direction; its allocation ticket never participates"
        )]
        let mut values = BTreeMap::<DerivedSourceKey, Arc<DerivedSourceObservation>>::new();
        let mut added = false;
        for (index, body) in original.iter().chain(bodies).enumerate() {
            let key = body.key();
            let names = key
                .relation()
                .len()
                .checked_add(key.identity().schema.0.len())
                .and_then(|n| n.checked_add(key.identity().name.len()))
                .and_then(|n| n.checked_add(128))
                .ok_or(Cause::WorkCounterOverflow)?;
            let levels = (usize::BITS - values.len().max(1).leading_zeros()) as usize;
            let work = names
                .checked_mul(11)
                .and_then(|n| n.checked_mul(levels))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Cause::WorkCounterOverflow)?;
            (if context.has_accounting() {
                context.checkpoint(work)
            } else {
                context.poll()
            })
            .map_err(Cause::from)?;
            if let Some(control) = control {
                control()?;
            }
            if let Some(existing) = values.get(key) {
                if !existing.same_truth_with_poll(body, || {
                    DerivedSourceObservation::checkpoint(context)?;
                    if let Some(control) = control {
                        control()?;
                    }
                    Ok::<_, Cause>(())
                })? {
                    return Err(Cause::ExpectedIdentitiesNotCanonical);
                }
                continue;
            }
            memory::grow_tree::<(DerivedSourceKey, Arc<DerivedSourceObservation>)>(
                &mut tree_memory,
                &mut tree_bytes,
                values.len(),
                execution,
                context.has_accounting(),
            )?;
            values.insert(key.clone(), Arc::clone(body));
            added |= index >= original.len();
        }
        if !added && let Some(prior) = prior {
            return Ok(Arc::clone(prior));
        }
        let bytes = values
            .len()
            .checked_mul(size_of::<Arc<DerivedSourceObservation>>())
            .and_then(|n| n.checked_add(size_of::<Self>() + 2 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = memory::reserve(execution, context.has_accounting(), bytes)?;
        let mut retained = Vec::new();
        retained
            .try_reserve_exact(values.len())
            .map_err(|_| Cause::AllocationFailed)?;
        for body in values.into_values() {
            DerivedSourceObservation::checkpoint(context)?;
            if let Some(control) = control {
                control()?;
            }
            retained.push(body);
        }
        Ok(Arc::new(Self {
            values: retained,
            _memory: memory,
        }))
    }
}
impl PartialEq for DerivedSourceDependencies {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for DerivedSourceDependencies {}
