//! Preserve original Source preparation causes and fund owned diagnostics.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProviderFailure,
};

pub(in crate::domain_computation::primary_graph::application_query) fn source_preparation_denial(
    execution: &ExecutionRequest<'_, '_>,
    cause: crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial,
) -> Denial {
    use {
        crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial as Work,
        crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
    };
    match cause {
        Work::Controlled(Stop::Content(cause)) => Denial::Content(cause),
        Work::Controlled(Stop::MaterialStorage(cause)) => source_storage(cause),
        Work::Controlled(Stop::Request(cause)) => source_storage(cause.into()),
        Work::Controlled(Stop::Storage(worth_execution::ExecutionRequestArrayDenial::Request(
            cause,
        ))) => source_storage(cause.into()),
        Work::Controlled(Stop::OrderedStorage(
            worth_execution::ExecutionOrderedMapDenial::Request(cause),
        )) => source_storage(cause.into()),
        Work::Controlled(Stop::NativeSource(cause)) => Denial::Source(
            worth_relational::facade::runtime::DerivedRelationReadDenial::Structural(cause),
        ),
        cause => retain_failure(execution, cause),
    }
}
#[derive(Debug)]
struct PreparationFailure<E: std::error::Error + Send + Sync + 'static> {
    cause: E,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<E: std::error::Error + Send + Sync + 'static> std::fmt::Display for PreparationFailure<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cause)
    }
}
impl<E: std::error::Error + Send + Sync + 'static> std::error::Error for PreparationFailure<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
pub(super) fn source_storage(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Denial {
    Denial::Source(worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause))
}
pub(in crate::domain_computation::primary_graph::application_query) fn retain_failure<
    E: std::error::Error + Send + Sync + 'static,
>(
    execution: &ExecutionRequest<'_, '_>,
    cause: E,
) -> Denial {
    if let Err(stop) = execution.consult() {
        return source_storage(stop.into());
    }
    let Some(bytes) = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<PreparationFailure<E>>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
    else {
        return source_storage(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow);
    };
    let memory = match execution.reserve_retained_memory(bytes) {
        Ok(memory) => memory,
        Err(stop) => return source_storage(stop.into()),
    };
    Denial::Provider(ManagedIndexSourceProviderFailure::from_owned(Arc::new(
        PreparationFailure {
            cause,
            _memory: memory,
        },
    )))
}
