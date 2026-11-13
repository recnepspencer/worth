//! The one World admission and actual A-to-B Native compilation.
use super::super::{
    refusal::WorthQueryApplicationHomeOpenRefused as OldRefused, runtime::DirectoryHome,
};
use super::refusal::{
    WorthQueryHomeProgramTransitionDenial as Denial,
    WorthQueryHomeProgramTransitionOpenRefused as Refused,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_contribution::WorthQueryBootstrapAdvancementPhase, application_home_integration::*,
};
use worth_store_application::{
    prepare_query_home_source, readmit_query_signal, readmit_query_world,
    ApplicationHomeWorldDefinition, UnsharedReadmittedApplicationHome,
};
pub(super) type SuccessorBinding = worth_proof::AdmittedApplicationHomeBinding<
    worth_query_installation::facade::WorthQueryHomeBindingAdmission,
>;

pub(super) struct ReadmittedHomeProgramTransition<
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
> {
    pub(super) before: PreparedExistingHomeSignalProgram<A, AP>,
    pub(super) target: PreparedHomeProgramTransitionTarget<B, BP>,
    pub(super) home: DirectoryHome,
    pub(super) source: worth_runtime_world::facade::ProductBranchObservation,
}
impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    ReadmittedHomeProgramTransition<A, AP, B, BP>
{
    pub(super) fn refused(self: Box<Self>, cause: Denial) -> Refused<A, AP, B, BP> {
        let Self {
            before,
            target,
            home,
            source: _,
        } = *self;
        Refused::transition(before, target, home, cause)
    }
}
pub(super) struct PreparedHomeProgramTransition<
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
> {
    pub(super) owners: Box<ReadmittedHomeProgramTransition<A, AP, B, BP>>,
    pub(super) binding: SuccessorBinding,
}
#[inline(never)]
pub(super) fn prepare<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    before: ReadmittedExistingHomeNativeMeaning<A, AP>,
    after: PreparedHomeApplication<B, BP>,
    home: UnsharedReadmittedApplicationHome,
) -> Result<PreparedHomeProgramTransition<A, AP, B, BP>, Refused<A, AP, B, BP>> {
    let request = phase.execution_request();
    let bridge = match before.home_bridge_definition() {
        Ok(bridge) => bridge,
        Err(cause) => {
            return Err(Refused::predecessor(
                OldRefused::ExistingBridgeDefinition {
                    query: before,
                    home,
                    cause,
                },
                after,
            ))
        }
    };
    let (budgets, clock, policy, execution, _conditionals) = after.home_world_resources();
    let source = match prepare_query_home_source(
        home.share(),
        ApplicationHomeWorldDefinition::new(bridge, "primary", budgets, clock, policy, execution),
        &request,
    ) {
        Ok(source) => source,
        Err(home) => {
            return Err(Refused::predecessor(
                OldRefused::ExistingWorldSourcePreparation {
                    query: before,
                    home,
                },
                after,
            ))
        }
    };
    let (registered, ordinary) = source.query_source_parts();
    let (before, builder, mut bindings) =
        match before.prepare_existing_home_signal_program(phase, registered, ordinary) {
            Ok(parts) => parts,
            Err(query) => {
                return Err(Refused::predecessor(
                    OldRefused::SignalProgramPreparation {
                        home: source,
                        query: Box::new(query),
                    },
                    after,
                ))
            }
        };
    let signal = match readmit_query_signal(source, builder, &mut bindings, &request) {
        Ok(signal) => signal,
        Err(home) => {
            return Err(Refused::predecessor(
                OldRefused::ExistingWorldReadmission {
                    query: before,
                    home,
                    _bindings: bindings,
                },
                after,
            ))
        }
    };
    let home = match readmit_query_world(signal, &request) {
        Ok(home) => home.consume(|home| home),
        Err(home) => {
            return Err(Refused::predecessor(
                OldRefused::ExistingWorldReadmission {
                    query: before,
                    home,
                    _bindings: bindings,
                },
                after,
            ))
        }
    };
    let source = match home
        .world()
        .observation_port()
        .observe_root_product_branch()
    {
        Ok(source) => source,
        Err(cause) => {
            return Err(Refused::before_compilation(
                before,
                after,
                home,
                Denial::Observation(cause),
            ))
        }
    };
    let binding = match after.admitted_home_binding() {
        Ok(binding) => binding,
        Err(cause) => {
            return Err(Refused::before_compilation(
                before,
                after,
                home,
                Denial::BindingAdmission(cause),
            ))
        }
    };
    let target = match after.prepare_home_program_transition_target(
        phase,
        home.relational().runtime(),
        &before,
        source.basis().relational_basis(),
    ) {
        Ok(target) => target,
        Err(successor) => return Err(Refused::compilation(before, successor, home)),
    };
    Ok(PreparedHomeProgramTransition {
        owners: Box::new(ReadmittedHomeProgramTransition {
            before,
            target,
            home,
            source,
        }),
        binding,
    })
}
