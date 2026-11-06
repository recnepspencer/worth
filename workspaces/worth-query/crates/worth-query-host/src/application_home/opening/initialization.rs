//! The Store origin gate alone can consume the original initializer.
use super::*;
#[inline(never)]
pub(super) fn initialize_directory<'authoring, Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    opening: preparation::DirectoryOpening<'authoring, Schema, Program>,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    let preparation::DirectoryOpening {
        query,
        home,
        checkpoint_limits,
        initial_state,
        observation,
    } = opening;
    let request = phase.execution_request();
    // Existing/Consumed origin never invokes the callback. Keep Query out
    // of its closure's ownership until Store has actually admitted it.
    let mut query = Some(query);
    // Retain the owning result in this caller. Store's initialization
    // callback carries only completion, keeping its generic phase frames
    // independent of Query's large refused-opening custody.
    let mut initialized = None;
    let opened = home.initialize_new_home(&request, |home| {
        initialized = Some(fresh::finish_new_home(
            phase,
            home,
            query
                .take()
                .expect("the one admitted recipe consumes Query once"),
            initial_state,
            checkpoint_limits,
            observation.clone(),
        ));
    });
    match opened {
        Ok(()) => initialized.expect("the one admitted initializer retained its result"),
        Err(home) => match home.reason() {
            ApplicationHomeInitializationDenial::ExistingHomeRequiresQueryPublication => {
                existing::finish_existing_home(
                    phase,
                    home.into_home(),
                    query.take().expect("existing origin retained Query"),
                    checkpoint_limits,
                    observation,
                )
            }
            ApplicationHomeInitializationDenial::Execution(_)
            | ApplicationHomeInitializationDenial::OpportunityAlreadyConsumed => {
                Err(Refused::QueryPublicationReadmissionRequired {
                    query: query.take().expect("refused origin retained Query"),
                    home,
                })
            }
        },
    }
}
