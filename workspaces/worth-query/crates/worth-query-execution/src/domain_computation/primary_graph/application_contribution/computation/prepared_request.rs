//! Sealed static compute transport; owner completion is a separate value.
use super::WorthQueryMemberExecution;
use std::any::Any;

pub(in crate::domain_computation::primary_graph) struct PreparedCompute {
    run: Box<dyn for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> ComputedSlot + Send>,
}

impl PreparedCompute {
    pub(in crate::domain_computation::primary_graph) fn new<T: Any + Send>(
        run: impl for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> T + Send + 'static,
    ) -> Self {
        Self {
            run: Box::new(move |member| ComputedSlot(Box::new(run(member)))),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn run(
        self,
        member: WorthQueryMemberExecution<'_>,
    ) -> ComputedSlot {
        (self.run)(member)
    }
}

pub(in crate::domain_computation::primary_graph) struct ComputedSlot(Box<dyn Any + Send>);
impl ComputedSlot {
    pub(in crate::domain_computation::primary_graph) fn into_value<T: Any + Send>(self) -> T {
        *self
            .0
            .downcast::<T>()
            .unwrap_or_else(|_| unreachable!("the sealed request fixes its result type"))
    }
}
