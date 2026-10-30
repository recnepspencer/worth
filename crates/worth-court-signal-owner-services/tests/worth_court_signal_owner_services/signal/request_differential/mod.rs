#[allow(unused_imports)]
use crate::request_differential::ordinary_context::*;
pub(crate) use worth_signal::facade::adapters::NodeContract;
pub(crate) use worth_signal::facade::specialist::ExecutionReport;
pub(crate) use {
    std::num::NonZeroUsize, std::sync::Condvar, std::sync::Mutex, std::sync::OnceLock,
    std::time::Duration,
};
pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionRequest, worth_execution::LeaseRequest,
    worth_execution::SerialMemoryBudget, worth_execution::SerialRequest,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};
pub(crate) use {
    worth_signal::facade::Aspect, worth_signal::facade::AspectVersion,
    worth_signal::facade::BoundedSignalInputs,
    worth_signal::facade::RunMode as EvaluationRequestMode, worth_signal::facade::SignalError,
    worth_signal::facade::SignalGraph,
};

#[test]
fn serial_default_admits_three_hundred_tasks_and_reports_their_charge() {
    let (values, report) = serial(300).unwrap();
    for (index, value) in values.iter().enumerate() {
        assert_eq!(
            value.get(worth_signal::facade::Aspect::new(0)),
            index as u64 + 1
        );
    }
    assert!(report.execution.last().unwrap().charged_work() > 0);
}

#[path = "request_custody/mod.rs"]
pub(crate) mod request_custody;
