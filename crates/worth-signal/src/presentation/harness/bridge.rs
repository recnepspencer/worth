mod adapter;
mod projection;
mod request;
mod translation;

use worth_harness::facade::{
    bench, parity_suite, ExecutionProfile, ExecutionRequest, HarnessBench, ParitySuite,
    ScenarioFixture,
};

use super::runtime::{SignalFixtureFactory, SignalMutationAction};

#[derive(Debug, Default, Clone, Copy)]
pub struct SignalHarnessBridge;

pub fn signal_bench(
    fixture: ScenarioFixture<SignalFixtureFactory>,
    request: ExecutionRequest<String>,
) -> HarnessBench<SignalHarnessBridge, SignalFixtureFactory, SignalMutationAction, String> {
    bench(SignalHarnessBridge, fixture, request)
}

pub fn signal_parity_suite(
    fixture: ScenarioFixture<SignalFixtureFactory>,
    request: ExecutionRequest<String>,
    baseline_profile: ExecutionProfile,
) -> ParitySuite<SignalHarnessBridge, SignalFixtureFactory, SignalMutationAction, String> {
    parity_suite(SignalHarnessBridge, fixture, request, baseline_profile)
}
