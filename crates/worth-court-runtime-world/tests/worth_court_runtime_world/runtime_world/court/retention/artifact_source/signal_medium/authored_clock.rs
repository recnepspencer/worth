use std::sync::Arc;
use worth_execution::time::{ControlledTime, TimeSampler};
use worth_execution::ExecutionRequest;
use worth_foundational::time::MonotonicInstant;
use worth_signal::facade::durability::{
    SignalAuthoredBranchInstallation, SignalAuthoredInstallationRefused,
    SignalAuthoredOwnerInstallation, SignalDurableMedium, SignalDurableTierVocabulary,
    SignalRecordedIdentityContribution,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

#[test]
fn authored_owner_readmission_keeps_the_receiver_clock() {
    let serial = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&serial);
    let (source, medium) = super::owner(&request);
    drop(source);
    let medium = medium.successor(&request);
    let sampler = Arc::new(TimeSampler::new(Arc::new(ControlledTime::at_unix_secs(1))));
    let program = Box::new(
        SignalRuntime::builder(SignalGraph::new())
            .with_time_sampler(sampler.clone())
            .with_kernel_defaults()
            .build(),
    );

    let mut observation = ReadmittedClock::default();
    let _owner = program
        .readmit_authored_owner(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(medium.binding().owner()),
            SignalDurableTierVocabulary::declare([((), "base")]).unwrap(),
            &mut observation,
            &request,
        )
        .unwrap();
    assert_eq!(observation.0, Some(sampler.monotonic()));
}

#[derive(Default)]
struct ReadmittedClock(Option<MonotonicInstant>);

impl SignalAuthoredOwnerInstallation for ReadmittedClock {
    fn readmit_branch(
        &mut self,
        branch: SignalAuthoredBranchInstallation<'_>,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalAuthoredInstallationRefused> {
        let observation = SignalRuntime::build_for::<()>(branch.graph().clone());
        self.0 = Some(observation.time_sampler().monotonic());
        Ok(())
    }
}
