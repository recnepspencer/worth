//! Actual Native/Bridge/Signal/World composition for filesystem Home courts.
mod definition;
mod publication;
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

#[derive(Debug)]
pub enum WorldHomeFixtureRefusal {
    Home(ApplicationHomeOpenRefusal),
    World(worth_store_application::WorldHomeReadmissionRefused),
}
impl WorldHomeFixture {
    pub fn begin_bootstrap_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> worth_runtime_world::facade::WorldDurableBootstrapOutcome {
        use worth_runtime_world::facade::ProductBranchCreationIntent;
        let native = self.home.relational().runtime();
        let (_, native_basis) = native
            .observe_branch(&native.main_branch_identity())
            .expect("actual Home Native owner observes its current root");
        let signal_basis = self
            ._signal
            .observe_signal_branch_basis(self._signal.current_branch())
            .expect("actual retained Signal owner observes its current root");
        self.home
            .world()
            .publication_port()
            .begin_durable_bootstrap_root(
                execution,
                ProductBranchCreationIntent::named("Original World Root").unwrap(),
                native_basis,
                signal_basis,
            )
    }
    pub fn bootstrap_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> worth_runtime_world::facade::PerformedRuntimeWorldBootstrap {
        use worth_runtime_world::facade::WorldDurableBootstrapOutcome as Outcome;
        let port = self.home.world().publication_port();
        let mut outcome = self.begin_bootstrap_root(execution);
        let backstop = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match outcome {
                Outcome::Performed(performed) => return performed,
                Outcome::Pending => {
                    execution
                        .consult()
                        .expect("original bootstrap request remains live");
                    assert!(
                        std::time::Instant::now() < backstop,
                        "actual World bootstrap did not settle"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    outcome = port.advance_durable_bootstrap_root(execution);
                }
                other => panic!("actual Home World bootstrap refused: {other:?}"),
            }
        }
    }
}

pub fn open(
    request: ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldHomeFixture, WorldHomeFixtureRefusal> {
    let (definition, signal) = definition::authored();
    let mut opened = None;
    let scope = execution.in_scope(|_| {
        opened = Some((|| {
            let recovered =
                recover_home(request, execution).map_err(WorldHomeFixtureRefusal::Home)?;
            let admitted =
                admit_home(recovered, execution).map_err(WorldHomeFixtureRefusal::Home)?;
            let native =
                readmit_relational(admitted, execution).map_err(WorldHomeFixtureRefusal::Home)?;
            let world = readmit_world(native, definition, execution)
                .map_err(WorldHomeFixtureRefusal::World)?
                .consume(|home| home);
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
        (Err(cause), _) => Err(WorldHomeFixtureRefusal::Home(
            ApplicationHomeOpenRefusal::Execution(cause),
        )),
        (Ok(()), None) => unreachable!("completed actual Home composition"),
    }
}
