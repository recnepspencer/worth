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
        let callback = move |member: WorthQueryMemberExecution<'_>| {
            let value = run(member);
            ComputedSlot {
                value: Box::new(value),
            }
        };
        Self {
            run: Box::new(callback),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn observe(
        self,
        before: impl Fn() + Send + Sync + 'static,
        after: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let callback = move |member: WorthQueryMemberExecution<'_>| {
            let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
            before();
            let result = self.run(member);
            after();
            result
        };
        Self {
            run: Box::new(callback),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn run(
        self,
        member: WorthQueryMemberExecution<'_>,
    ) -> ComputedSlot {
        let _custody = super::worker_custody::MemberComputeCustody::enter();
        (self.run)(member)
    }
}

pub(in crate::domain_computation::primary_graph) struct ComputedSlot {
    value: Box<dyn Any + Send>,
}
impl ComputedSlot {
    pub(in crate::domain_computation::primary_graph) fn into_value<T: Any + Send>(self) -> T {
        *self
            .value
            .downcast::<T>()
            .unwrap_or_else(|_| unreachable!("the sealed request fixes its result type"))
    }
}

#[cfg(test)]
impl PreparedCompute {
    pub(in crate::domain_computation::primary_graph) fn declared_test_request(
        rank: usize,
        fails: bool,
    ) -> Self {
        Self::new(move |member| {
            let (_, poll) = member.into_parts();
            poll().unwrap();
            (rank, fails)
        })
    }
}
