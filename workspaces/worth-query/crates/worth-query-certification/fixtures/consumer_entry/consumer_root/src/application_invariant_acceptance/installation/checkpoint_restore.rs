//! Genuine native reconstruction reuses installed meaning and checkpointed records.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationProgramDefinition, ValidatedApplicationProgram,
};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

pub(in crate::application_invariant_acceptance) fn restore_selected_program<P>(
    world: &ConsumerWorld<P>,
    program: ValidatedApplicationProgram<ConsumerSchema, P>,
    checkpoint: installation::WorthQueryApplicationCheckpoint,
) -> installation::WorthQueryProgramApplicationRuntime<ConsumerSchema, P>
where
    P: ApplicationProgramDefinition<ConsumerSchema,
        Contributions = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions>,
    P::Outputs: installation::WorthQueryApplicationProgramRoots<ConsumerSchema>,
{
    let configuration = (
        TopologyConfiguration {
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::clone(&world.invariant_calls),
            invariant_probe: Arc::clone(&world.invariant_probe),
            producer_authorization_denials: Arc::clone(&world.producer_authorization_denials),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
        },
        Arc::new(AtomicUsize::new(0)),
        (),
    );
    installation::in_memory_rostered_program_from_checkpoint(
        program,
        installation::WorthQueryApplicationProgramRoster::new(),
        ConsumerSchema::declaration().unwrap(),
        configuration,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        resources::application_limits(),
        checkpoint,
    )
    .expect("the authentic checkpoint restores under the identical installed program")
}
