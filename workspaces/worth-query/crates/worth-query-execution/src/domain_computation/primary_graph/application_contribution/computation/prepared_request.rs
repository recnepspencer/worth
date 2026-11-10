//! Sealed static compute transport; owner completion is a separate value.
use super::WorthQueryMemberExecution;
use crate::domain_computation::execution_runtime::product_world::published_effect::PublishedEffect;
use std::any::Any;

pub struct PreparedCompute {
    run: Box<dyn for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> ComputedSlot + Send>,
}

impl PreparedCompute {
    pub(in crate::domain_computation::primary_graph) fn new<T: Any + Send>(
        run: impl for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> T + Send + 'static,
    ) -> Self {
        // A member may run on a descendant worker and commit there. It carries
        // its advancement's published-effect marker (public #132), so World's
        // outcome reaches the runner from that thread too.
        let published = PublishedEffect::current();
        let callback = move |member: WorthQueryMemberExecution<'_>| {
            let _published = PublishedEffect::install(published);
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

pub struct ComputedSlot {
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

mod court_doors_for_test;
