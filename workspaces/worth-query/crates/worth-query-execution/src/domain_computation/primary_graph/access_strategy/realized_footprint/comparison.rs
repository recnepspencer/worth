//! Erased comparisons cannot swallow the caller's exact original failure.
use std::cell::RefCell;
use worth_relational::facade::content::RelationalContentFrame;

struct FirstFailure<E>(RefCell<Option<E>>);
impl<E> FirstFailure<E> {
    fn new() -> Self {
        Self(RefCell::new(None))
    }
    fn run<T>(&self, operation: impl FnOnce() -> Result<T, E>) -> Result<T, ()> {
        if self.0.borrow().is_some() {
            return Err(());
        }
        operation().map_err(|cause| {
            *self.0.borrow_mut() = Some(cause);
        })
    }
    fn finish(self, equal: bool, admit: &mut impl FnMut(u64) -> Result<(), E>) -> Result<bool, E> {
        match self.0.into_inner() {
            Some(cause) => Err(cause),
            None => {
                admit(0)?;
                Ok(equal)
            }
        }
    }
}

pub(super) fn with_checkpoint<E>(
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    compare: &mut impl FnMut(&mut dyn FnMut(u64) -> bool) -> bool,
) -> Result<bool, E> {
    let failure = FirstFailure::new();
    let equal = compare(&mut |work| failure.run(|| admit(work)).is_ok());
    failure.finish(equal, admit)
}
#[cfg(test)]
mod tests;

/// All erased channels share the first original failure. Scratch must
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
    frames: &mut dyn FnMut(&RelationalContentFrame, &RelationalContentFrame) -> Result<bool, E>,
) -> Result<bool, E> {
    let failure = FirstFailure::new();
    let mut checkpoint = |work| failure.run(|| admit(work)).is_ok();
    let mut reservation = |bytes| failure.run(|| reserve(bytes));
    let mut storage_fault = |cause| {
        let _ = failure.run(|| Err::<(), _>(fault(cause)));
    };
    let mut frame_comparison = |left: &RelationalContentFrame, right: &RelationalContentFrame| {
        failure.run(|| frames(left, right))
    };
    let equal = compare(
        &mut checkpoint,
        &mut crate::domain_computation::primary_graph::application_query::ContentScratch {
            frames: &mut frame_comparison,
            reserve: &mut reservation,
            fault: &mut storage_fault,
        },
    );
    failure.finish(equal, admit)
}
