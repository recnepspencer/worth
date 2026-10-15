//! Admission for the stamp's actual nested vectors and owned locator text.
use super::*;
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub(super) struct DependencyStorage {
    _funding: Option<ExecutionMemoryReservation>,
}

#[derive(Default)]
pub(super) struct StorageQuote {
    bytes: usize,
    dependencies: usize,
}

impl StorageQuote {
    pub(super) fn add_vector<T>(
        &mut self,
        count: usize,
    ) -> Result<(), RelationalAuthorizationDependencyDenial> {
        self.add_terms(count)?;
        self.add_bytes(
            count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(RelationalAuthorizationDependencyDenial::ByteBudgetExceeded)?,
        )
    }
    pub(super) fn add_terms(
        &mut self,
        count: usize,
    ) -> Result<(), RelationalAuthorizationDependencyDenial> {
        self.dependencies = self
            .dependencies
            .checked_add(count)
            .filter(|count| *count <= MAXIMUM_AUTHORIZATION_DEPENDENCIES)
            .ok_or(RelationalAuthorizationDependencyDenial::DependencyBudgetExceeded)?;
        Ok(())
    }
    pub(super) fn add_bytes(
        &mut self,
        bytes: usize,
    ) -> Result<(), RelationalAuthorizationDependencyDenial> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(RelationalAuthorizationDependencyDenial::ByteBudgetExceeded)?;
        Ok(())
    }
    pub(super) fn merge(
        &mut self,
        other: Self,
    ) -> Result<(), RelationalAuthorizationDependencyDenial> {
        self.dependencies = self
            .dependencies
            .checked_add(other.dependencies)
            .filter(|count| *count <= MAXIMUM_AUTHORIZATION_DEPENDENCIES)
            .ok_or(RelationalAuthorizationDependencyDenial::DependencyBudgetExceeded)?;
        self.add_bytes(other.bytes)
    }
    pub(super) fn admit(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<DependencyStorage, RelationalAuthorizationDependencyDenial> {
        use RelationalAuthorizationDependencyDenial as Denial;
        execution
            .check_live()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        let bytes = u64::try_from(self.bytes).map_err(|_| Denial::ByteBudgetExceeded)?;
        let held = match execution.allocation_policy() {
            ExecutionAllocationPolicy::SystemAllocation => None,
            ExecutionAllocationPolicy::Execution(lease) => {
                Some(lease.reserve_memory(bytes).map_err(|cause| {
                    Denial::Execution(crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    ))
                })?)
            }
        };
        execution
            .check_live()
            .map_err(|stop| Denial::Execution(stop.into()))?;
        Ok(DependencyStorage { _funding: held })
    }
}

pub(super) fn capture_quote(
    evidence: &RelationalAuthorizationObservationEvidence,
) -> Result<StorageQuote, RelationalAuthorizationDependencyDenial> {
    use RelationalAuthorizationDependencyDenial as Denial;
    let mut quote = StorageQuote::default();
    quote.add_terms(2)?;
    quote.add_vector::<PathDependencies>(evidence.paths().len())?;
    let mut text = 0usize;
    for path in evidence.paths() {
        if !path.exhaustive() || path.matched() != path.witness().is_some() {
            return Err(Denial::IncompleteObservation);
        }
        quote
            .add_vector::<EntityId>(path.witness().map_or(0, |witness| witness.entities().len()))?;
        quote.add_vector::<EntityDependency>(path.entities().len())?;
        quote.add_vector::<RelationDependency>(path.relations().len())?;
        quote.add_vector::<AdjacencyDependency>(path.adjacency_lists().len())?;
        quote.add_vector::<FieldDependency>(path.fields().len())?;
        for (_, locator) in path.fields() {
            let [field] = locator.field_path().fields() else {
                return Err(Denial::UnsupportedFieldLocator);
            };
            text = text
                .checked_add(locator.aspect().aspect_key().as_str().len())
                .and_then(|sum| sum.checked_add(field.as_str().len()))
                .filter(|sum| *sum <= MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES)
                .ok_or(Denial::ByteBudgetExceeded)?;
        }
    }
    quote.add_bytes(text)?;
    Ok(quote)
}

/// The input cardinality was admitted as one exact Vec allocation before capture.
pub(super) fn capture_list<T>(
    count: usize,
    values: impl Iterator<Item = Result<T, RelationalAuthorizationDependencyDenial>>,
) -> Result<Vec<T>, RelationalAuthorizationDependencyDenial> {
    let mut result = Vec::with_capacity(count);
    for value in values {
        result.push(value?);
    }
    Ok(result)
}

pub(super) fn locator_storage(
    execution: &ExecutionRequest<'_, '_>,
    aspect: &str,
    field: &str,
) -> Result<DependencyStorage, RelationalAuthorizationDependencyDenial> {
    let mut quote = StorageQuote::default();
    quote.add_bytes(aspect.len())?;
    quote.add_bytes(field.len())?;
    quote.add_bytes(std::mem::size_of::<worth_foundational::facade::FieldKey>())?;
    quote.admit(execution)
}
