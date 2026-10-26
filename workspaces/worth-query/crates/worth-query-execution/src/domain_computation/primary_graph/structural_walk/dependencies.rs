//! One funded descriptor list shared by a path's complete-walk consumers.
use super::*;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct StructuralWalkDependencies {
    values: Vec<Arc<StructuralWalkObservation>>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl StructuralWalkDependencies {
    pub(in crate::domain_computation::primary_graph) fn values(
        &self,
    ) -> &[Arc<StructuralWalkObservation>] {
        &self.values
    }

    pub(in crate::domain_computation::primary_graph) fn append(
        prior: Option<&Arc<Self>>,
        body: &Arc<StructuralWalkObservation>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Arc<Self>, Cause> {
        let values = prior.map_or(&[][..], |p| p.values());
        for existing in values {
            StructuralWalkObservation::checkpoint(context)?;
            if existing.key() == body.key() {
                if !existing
                    .same_truth_with_poll(body, || StructuralWalkObservation::checkpoint(context))?
                {
                    return Err(Cause::ExpectedIdentitiesNotCanonical);
                }
                return Ok(Arc::clone(
                    prior.expect("an existing dependency has an owner"),
                ));
            }
        }
        let count = values
            .len()
            .checked_add(1)
            .ok_or(Cause::ChargedBytesOverflow)?;
        let bytes = count
            .checked_mul(size_of::<Arc<StructuralWalkObservation>>())
            .and_then(|n| n.checked_add(size_of::<Self>() + 2 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = memory::reserve(execution, context.has_accounting(), bytes)?;
        let mut retained = Vec::new();
        retained
            .try_reserve_exact(count)
            .map_err(|_| Cause::AllocationFailed)?;
        for value in values {
            StructuralWalkObservation::checkpoint(context)?;
            retained.push(Arc::clone(value));
        }
        retained.push(Arc::clone(body));
        StructuralWalkObservation::checkpoint(context)?;
        Ok(Arc::new(Self {
            values: retained,
            _memory: memory,
        }))
    }
}

impl PartialEq for StructuralWalkDependencies {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for StructuralWalkDependencies {}
