//! Actual Native/Bridge/Signal/World composition for filesystem Home courts.
mod definition;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};
use worth_signal::facade::SignalRuntime;
use worth_store_application::{
    admit_home, readmit_relational, readmit_world, recover_home, ApplicationHomeOpenRefusal,
    ApplicationHomeRequest, WorldReadmittedApplicationHome,
};

pub struct WorldHomeFixture {
    pub home: WorldReadmittedApplicationHome<(), (), (), (), ()>,
    _signal: SignalRuntime<(), (), (), (), ()>,
}

pub fn open(
    request: ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldHomeFixture, ApplicationHomeOpenRefusal> {
    let (definition, signal) = definition::authored();
    let mut opened = None;
    let scope = execution.in_scope(|_| {
        opened = Some((|| {
            let recovered = recover_home(request, execution)?;
            let admitted = admit_home(recovered, execution)?;
            let native = readmit_relational(admitted, execution)?;
            let world = readmit_world(native, definition, execution)?.consume(|home| home);
            Ok(WorldHomeFixture {
                home: world,
                _signal: signal,
            })
        })());
    });
    match (scope, opened) {
        (Ok(()), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(ApplicationHomeOpenRefusal::Execution(cause)),
        (Ok(()), None) => unreachable!("completed actual Home composition"),
    }
}
