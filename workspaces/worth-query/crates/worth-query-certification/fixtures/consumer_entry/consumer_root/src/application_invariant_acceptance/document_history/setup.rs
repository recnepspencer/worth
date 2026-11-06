//! Finite installation and initial output warmup precede transfer of model heads.
use super::*;
use model::with_request;

pub(super) fn install(
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) -> installation::ConsumerWorld<FixedProgram> {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, FixedProgram>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().unwrap())
        .validated_program()
        .unwrap();
    // The executable owns two documents, four authored futures, transient
    // checkout heads and a two-member group under explicit finite admission.
    let world = installation::install_selected_program_with_limits(
        installed,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        false,
        super::super::resources::application_limits_with_branches(32),
        program,
    )
    .unwrap();
    warm(&world.application, "anchor-a");
    warm(&world.application, "remote-a");
    world
}

fn warm(app: &Runtime, body: &str) {
    let mut operation = with_request(app, |request| {
        request
            .demand(PlanarOutputDemand::new(body))
            .retain_program_outputs::<FixedProgram, Root>(app)
            .unwrap_or_else(|cause| panic!("initial required output: {:?}", cause.denial()))
    });
    for _ in 0..96 {
        operation = match with_request(app, |request| operation.advance(request)) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                with_request(app, |request| {
                    settled.close(request).unwrap_or_else(|cause| {
                        panic!("initial output cleanup: {:?}", cause.denial())
                    })
                });
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("initial output refusal: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("initial output terminal: {:?}", terminal.denial())
            }
        };
    }
    panic!("finite required output graph did not settle");
}

pub(super) fn configure<'r>(document: Document<'r>, app: &Runtime) -> Configured<'r> {
    app.with_host_advancement(|phase| {
        with_request(app, |request| {
            document
                .with_outputs::<FixedProgram, Root>(
                    &phase,
                    request,
                    app,
                    PlanarOutputDemand::new("anchor-a"),
                    Default::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("required graph configuration: {cause:?}"))
        })
    })
    .unwrap()
}
