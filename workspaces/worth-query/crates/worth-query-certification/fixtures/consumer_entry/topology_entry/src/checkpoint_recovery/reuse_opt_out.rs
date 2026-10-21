//! Live outputs whose Initial producer does not promise decision reuse.

use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryProducerInputReuseContract, WorthQueryProducerLifecyclePosture,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandProgress;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

mod build_identity;
mod every_consult;
mod program;
mod required_refresh;
use program::{FinalConnection, NoReuseProgram};

worth_query_application! {
    NoReuseSchema {
        owner: "worth.query.certification.no-input-reuse",
        version: (1, 0),
        contributions: [TopologyContribution],
    }
}

impl TopologySchemaBinding for NoReuseSchema {
    const ROOT_INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = None;
    const FINAL_INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> = None;
}

#[test]
fn live_output_without_initial_input_reuse_selects_preserve_and_keeps_its_entity() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_no_reuse(None);
    check_live_output(&application);
}

fn install_no_reuse(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
) -> application_installation::WorthQueryProgramApplicationRuntime<NoReuseSchema, NoReuseProgram> {
    install_no_reuse_with_build(
        checkpoint,
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
    )
}

fn install_no_reuse_with_build(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    build_identity: ApplicationBuildIdentity,
) -> application_installation::WorthQueryProgramApplicationRuntime<NoReuseSchema, NoReuseProgram> {
    let program = ApplicationProgramAuthoring::<NoReuseSchema, NoReuseProgram>::begin()
        .validated_program()
        .unwrap();
    let configuration = (TopologyConfiguration {
        final_feature: crate::planar_final_feature_runtime(),
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: Arc::new(AtomicBool::new(false)),
    },);
    // Window <= history is enforced by primary_graph/bootstrap/preparation.rs:91.
    let limits = support::limits(32, support::invalidation());
    if let Some(checkpoint) = checkpoint {
        return application_installation::in_memory_program_from_checkpoint(
            program,
            NoReuseSchema::declaration().unwrap(),
            configuration,
            build_identity,
            limits,
            checkpoint,
        )
        .unwrap();
    }
    application_installation::in_memory_program(
        program,
        NoReuseSchema::declaration().unwrap(),
        configuration,
        build_identity,
        limits,
        |_phase, graph, installed| {
            let principal = installed.principal_binding(
                ConsumerPrincipalBinding::reference::<NoReuseSchema>(),
            ).unwrap();
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                support::external_identity(),
                worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            support::seed_cycle(graph);
            Ok(())
        },
    ).unwrap()
}

fn check_live_output(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        NoReuseSchema,
        NoReuseProgram,
    >,
) {
    let (scope, principal) = support::authenticate(application);
    let request = application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        crate::checkpoint_recovery::support::host_call(application, |phase| {
            application
                .select_output_producer::<PlanarFinalOutputFamily>(
                    phase,
                    &source.observed_sources()[0],
                    "planar-final",
                )
                .map_err(Box::new)
        })
        .unwrap()
        .applicability()
        .lifecycle(),
        WorthQueryProducerLifecyclePosture::Initial
    );
    drop(source);

    let mut initial = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(application)
        .unwrap();
    let first = (0..64)
        .find_map(|_| match initial.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the Initial producer creates the final ring");
    let entity = first
        .outputs_of::<FinalPlanarOutputs>()
        .unwrap()
        .entity::<FinalAnchorOutput<NoReuseSchema>>()
        .unwrap()
        .entity_id();
    drop((first, initial));

    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let selected = crate::checkpoint_recovery::support::host_call(application, |phase| {
        application
            .select_output_producer::<PlanarFinalOutputFamily>(
                phase,
                &source.observed_sources()[0],
                "planar-final",
            )
            .map_err(Box::new)
    })
    .unwrap();
    assert_eq!(
        selected.applicability().lifecycle(),
        WorthQueryProducerLifecyclePosture::Preserve
    );
    drop(source);

    let mut preserving = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(application)
        .unwrap();
    let second = (0..64)
        .find_map(|_| match preserving.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the Preserve producer advances without Initial executing over live output");
    assert_eq!(second.producer_contacts_in_this_demand(), 1);
    assert_eq!(
        second
            .outputs_of::<FinalPlanarPreserveOutputs>()
            .unwrap()
            .entity::<FinalPreservedAnchorOutput<NoReuseSchema>>()
            .unwrap()
            .entity_id(),
        entity
    );
}
