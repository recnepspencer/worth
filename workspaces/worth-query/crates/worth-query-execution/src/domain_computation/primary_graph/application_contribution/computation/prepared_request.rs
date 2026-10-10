//! Sealed static compute transport; owner completion is a separate value.
use super::WorthQueryMemberExecution;
use std::any::Any;
use worth_execution::ChargedBytes;

pub(in crate::domain_computation::primary_graph) struct PreparedCompute {
    run: Box<dyn for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> ComputedSlot + Send>,
    input_heap: u64,
    result_capacity: u64,
}

impl PreparedCompute {
    pub(in crate::domain_computation::primary_graph) fn new<T: Any + Send>(
        owned_heap: u64,
        result_heap_capacity: u64,
        run: impl for<'a> FnOnce(WorthQueryMemberExecution<'a>) -> T + Send + 'static,
        result_heap: fn(&T) -> u64,
    ) -> Self {
        let callback = move |member: WorthQueryMemberExecution<'_>| {
            let value = run(member);
            let bytes = (size_of::<T>() as u64).saturating_add(result_heap(&value));
            ComputedSlot {
                value: Box::new(value),
                bytes,
            }
        };
        let input_heap = (size_of_val(&callback) as u64).saturating_add(owned_heap);
        Self {
            run: Box::new(callback),
            input_heap,
            result_capacity: (size_of::<T>() as u64).saturating_add(result_heap_capacity),
        }
    }

    #[cfg(feature = "test-query-execution-observer")]
    pub(in crate::domain_computation::primary_graph) fn observe(
        self,
        before: impl Fn() + Send + Sync + 'static,
        after: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let input_heap = self.input_heap;
        let result_capacity = self.result_capacity;
        let callback = move |member: WorthQueryMemberExecution<'_>| {
            before();
            let result = self.run(member);
            after();
            result
        };
        Self {
            input_heap: input_heap.saturating_add(size_of_val(&callback) as u64),
            result_capacity,
            run: Box::new(callback),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn result_capacity(&self) -> u64 {
        self.result_capacity
    }

    pub(in crate::domain_computation::primary_graph) fn run(
        self,
        member: WorthQueryMemberExecution<'_>,
    ) -> ComputedSlot {
        let _custody = super::worker_custody::MemberComputeCustody::enter();
        (self.run)(member)
    }
}

impl ChargedBytes for PreparedCompute {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            run: _,
            input_heap,
            result_capacity: _,
        } = self;
        *input_heap
    }
}

pub(in crate::domain_computation::primary_graph) struct ComputedSlot {
    value: Box<dyn Any + Send>,
    bytes: u64,
}
impl ComputedSlot {
    pub(in crate::domain_computation::primary_graph) fn into_value<T: Any + Send>(self) -> T {
        *self
            .value
            .downcast::<T>()
            .unwrap_or_else(|_| unreachable!("the sealed request fixes its result type"))
    }
}
impl ChargedBytes for ComputedSlot {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { value: _, bytes } = self;
        *bytes
    }
}

#[cfg(test)]
impl PreparedCompute {
    pub(in crate::domain_computation::primary_graph) fn declared_test_request(
        rank: usize,
        units: u64,
        fails: bool,
    ) -> Self {
        Self::new(
            0,
            0,
            move |member| {
                let (_, mut charge) = member.into_parts();
                charge.as_mut().unwrap()(units).unwrap();
                (rank, fails)
            },
            |_| 0,
        )
    }
}
