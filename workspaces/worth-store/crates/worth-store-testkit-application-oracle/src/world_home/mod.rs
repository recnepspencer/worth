//! Actual Native/Bridge/Signal/World composition for filesystem Home courts.
mod definition;
mod derived_signal;
pub use derived_signal::{open_with_derived_signal, reference_signal_value};
mod publication;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_store_application::{
    admit_home, readmit_relational, readmit_world, recover_home, ApplicationHomeOpenRefusal,
    ApplicationHomeRequest, WorldReadmittedApplicationHome,
};

pub struct WorldHomeFixture {
    pub home: WorldReadmittedApplicationHome<(), (), (), (), ()>,
}

pub enum WorldHomeFixtureRefusal {
    Home(Box<ApplicationHomeOpenRefusal>),
    Readmission(Box<worth_store_application::RelationalHomeOpenRefused>),
    Admitted {
        fixture: Box<WorldHomeFixture>,
        cause: WorkCeilingDenial,
    },
    World(Box<worth_store_application::WorldHomeReadmissionRefused>),
}
impl std::fmt::Debug for WorldHomeFixtureRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Home(cause) => f.debug_tuple("Home").field(cause).finish(),
            Self::Readmission(home) => f.debug_tuple("Readmission").field(home).finish(),
            Self::World(home) => f.debug_tuple("World").field(home).finish(),
            Self::Admitted { fixture, cause } => f
                .debug_struct("Admitted")
                .field("home", &fixture.home.home_identity())
                .field("cause", cause)
                .finish(),
        }
    }
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
        let source = self
            .home
            .standalone_signal_source()
            .expect("the Home retains its actual standalone Signal owner");
        let (identity, _) = source
            .branches()
            .expect("actual authored branches")
            .into_iter()
            .next()
            .expect("installed original branch");
        let descriptor = source
            .observe(identity)
            .expect("actual authored branch descriptor");
        let signal_basis = source
            .readmit_descriptor(&descriptor)
            .expect("same actual Signal owner")
            .basis()
            .clone();
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
    let definition = definition::authored();
    open_definition(request, execution, definition)
}

pub fn open_with_retained_history_limit(
    request: ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<WorldHomeFixture, WorldHomeFixtureRefusal> {
    let definition = definition::authored_with_history_limit(limit);
    open_definition(request, execution, definition)
}

struct HomeCompositionInputs {
    request: ApplicationHomeRequest,
    definition: worth_store_application::ApplicationWorldDefinition<(), (), (), (), ()>,
}

fn open_definition(
    request: ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
    definition: worth_store_application::ApplicationWorldDefinition<(), (), (), (), ()>,
) -> Result<WorldHomeFixture, WorldHomeFixtureRefusal> {
    // Scope callbacks carry pointers to the original owning inputs, rather
    // than copying a large source owner through every execution wrapper.
    let inputs = Box::new(HomeCompositionInputs {
        request,
        definition,
    });
    let mut opened = None;
    let scope = execution.in_scope(|_| {
        opened = Some(compose(inputs, execution));
    });
    match (scope, opened) {
        (Ok(()), Some(result)) => result,
        (Err(_), Some(Err(refused))) => Err(refused),
        (Err(cause), Some(Ok(fixture))) => Err(WorldHomeFixtureRefusal::Admitted {
            fixture: Box::new(fixture),
            cause,
        }),
        (Err(cause), None) => Err(WorldHomeFixtureRefusal::Home(Box::new(
            ApplicationHomeOpenRefusal::Execution(cause),
        ))),
        (Ok(()), None) => unreachable!("completed actual Home composition"),
    }
}

#[inline(never)]
fn compose(
    inputs: Box<HomeCompositionInputs>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldHomeFixture, WorldHomeFixtureRefusal> {
    let HomeCompositionInputs {
        request,
        definition,
    } = *inputs;
    let recovered = recover_home(request, execution)
        .map_err(|cause| WorldHomeFixtureRefusal::Home(Box::new(cause)))?;
    let admitted = admit_home(recovered, execution)
        .map_err(|cause| WorldHomeFixtureRefusal::Home(Box::new(cause)))?;
    let native = readmit_relational(admitted, execution)
        .map_err(|cause| WorldHomeFixtureRefusal::Readmission(Box::new(cause)))?;
    let world = readmit_world(native, definition, execution)
        .map_err(|cause| WorldHomeFixtureRefusal::World(Box::new(cause)))?
        .consume(|home| home);
    Ok(WorldHomeFixture { home: world })
}
