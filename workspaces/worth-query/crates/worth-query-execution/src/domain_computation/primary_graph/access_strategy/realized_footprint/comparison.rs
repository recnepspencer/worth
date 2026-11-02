//! Erased comparisons cannot swallow the caller's exact original failure.
pub(super) fn with_checkpoint<E>(
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    compare: &mut impl FnMut(&mut dyn FnMut(u64) -> bool) -> bool,
) -> Result<bool, E> {
    let mut failure = None;
    let equal = compare(&mut |work| {
        if failure.is_some() {
            return false;
        }
        match admit(work) {
            Ok(()) => true,
            Err(cause) => {
                failure = Some(cause);
                false
            }
        }
    });
    match failure {
        Some(cause) => Err(cause),
        None => {
            admit(0)?;
            Ok(equal)
        }
    }
}
#[cfg(test)]
mod tests;

/// All three erased channels share the first original failure. Scratch must
/// release its vector before releasing the returned original reservation.
pub(super) fn with_resources<E>(
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    reserve: &mut impl FnMut(u64) -> Result<Option<worth_execution::ExecutionMemoryReservation>, E>,
    fault: &mut impl FnMut(
        crate::domain_computation::primary_graph::application_query::ProjectionContentComparisonFault,
    ) -> E,
    compare: &mut impl FnMut(
        &mut dyn FnMut(u64) -> bool,
        &mut crate::domain_computation::primary_graph::application_query::ContentScratch<'_>,
    ) -> bool,
) -> Result<bool, E> {
    use std::cell::RefCell;
    let failure = RefCell::new(None);
    let mut checkpoint = |work| {
        if failure.borrow().is_some() {
            return false;
        }
        match admit(work) {
            Ok(()) => true,
            Err(cause) => {
                *failure.borrow_mut() = Some(cause);
                false
            }
        }
    };
    let mut reservation = |bytes| {
        if failure.borrow().is_some() {
            return Err(());
        }
        reserve(bytes).map_err(|cause| {
            *failure.borrow_mut() = Some(cause);
        })
    };
    let mut storage_fault = |cause| {
        if failure.borrow().is_none() {
            *failure.borrow_mut() = Some(fault(cause));
        }
    };
    let equal = compare(
        &mut checkpoint,
        &mut crate::domain_computation::primary_graph::application_query::ContentScratch {
            reserve: &mut reservation,
            fault: &mut storage_fault,
        },
    );
    match failure.into_inner() {
        Some(cause) => Err(cause),
        None => {
            admit(0)?;
            Ok(equal)
        }
    }
}
