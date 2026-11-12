//! One configured clock for both World limits and current authorization.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
struct TimeSource(WorthQueryAdmissionClock);
impl WorthQueryRuntimeTimeSource for TimeSource {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        self.0.clone()
    }
    fn current_time(&self) -> Result<std::time::SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::SystemTime::now())
    }
}
pub(in super::super) fn install<P>(
    clock: WorthQueryAdmissionClock,
    worker: bool,
    value: u64,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let schema = Schema::declaration().unwrap();
    in_memory_program_with_authorization_time_source(
        ApplicationProgramAuthoring::<Schema, P>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        (Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        if worker {
            worker_limits_with_clock(clock.clone())
        } else {
            host_limits(clock.clone())
        },
        |_, graph, installed| seed(graph, installed, value, None),
        TimeSource(clock),
    )
    .unwrap()
}
