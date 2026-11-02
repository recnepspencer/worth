//! A refused real attachment must retain the configured executable instances.
use super::*;
use crate::domain_computation::primary_graph::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_declaration::facade::application_program::*;
#[path = "fixture.rs"]
mod fixture;
use fixture::{Catalog, Program, Provider};
#[derive(Debug)]
struct MaintenanceMustRemainUnentered;
impl crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance
    for MaintenanceMustRemainUnentered
{
    fn after_advancement(&self, _: &worth_execution::ExecutionRequest<'_, '_>) {
        panic!("the refused Memory-root attachment must never enter Directory maintenance");
    }
}

#[test]
fn refused_home_world_attachment_retains_nonempty_provider_instances() {
    with_test_advancement(|active| {
        let phase = active.bootstrap_for_test();
        let drops = Arc::new(AtomicUsize::new(0));
        let limits = WorthQueryInMemoryApplicationLimits::new(
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
            crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile::default(),
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        );
        let declaration = Catalog::declaration().unwrap();
        let PreparedHomeApplication { state, invariants } = prepare_home_program(
            ApplicationProgramAuthoring::<Catalog, Program>::begin()
                .with_schema(&declaration)
                .validated_program()
                .unwrap(),
            WorthQueryApplicationProgramRoster::new(),
            declaration,
            (Provider(Arc::clone(&drops)),),
            worth_foundational::facade::ApplicationBuildIdentity::parse(
                "71717171-7171-7171-7171-717171717171",
            )
            .unwrap(),
            limits,
        )
        .unwrap_or_else(|cause| panic!("real installed meaning: {cause:?}"));
        let mut bootstrap = state
            .authority
            .prepare_primary_graph_with_relational_runtime_and_invariants(
                &phase,
                &state.runtime,
                &state.installed,
                worth_relational::facade::runtime::RelationalRuntimeApi::builder()
                    .build()
                    .unwrap(),
                state.limits.world.clone(),
                invariants,
            )
            .unwrap();
        let mut state = state;
        bootstrap.managed_indexes = std::mem::take(&mut state.pending.managed_indexes);
        bootstrap.program_activation = Some(state.activation.clone());
        bootstrap.program_activation_seed = Some(
            super::super::super::super::bootstrap::WorthQueryProgramActivationSeed::for_initial_program(
                &state.support.initial_revision, state.activation.clone(),
            ),
        );
        fixture::seed_principal(&mut bootstrap, &state.installed);
        let bridge = super::super::super::super::managed_bridge::install_application_bridge(
            &state.installed,
            &bootstrap.graph.layout,
            bootstrap
                .graph
                .integration_handle()
                .relational_bridge_source(),
            state.limits.conditionals,
        )
        .unwrap()
        .seal()
        .unwrap();
        let publication = bootstrap.publish_home_bootstrap_rows(&phase).unwrap();
        let graph = bootstrap.graph.integration_handle();
        let branch = graph.with_runtime(&phase.execution_request(), |native| {
            native.main_branch_identity()
        });
        let relational = graph
            .prepare_product_source(&phase.execution_request(), &branch)
            .unwrap();
        // A genuine memory World is not a directory World. Its actual root
        // reaches the production attachment's explicit durable-owner refusal.
        let world = WorthQueryProductRuntime::install(
            relational,
            &mut bridge.conditional_lifecycle(),
            state.limits.world.clone(),
            None,
        )
        .unwrap();
        let root = world
            .owner
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        let prepared = PreparedHomeQueryRuntime {
            prepared: HomeQueryPreparation {
                state,
                bootstrap,
                bridge,
            },
            publication: Some(publication),
        };
        let instances = prepared
            .prepared
            .bootstrap
            .managed_indexes
            .configured_instances_for_test();
        assert_eq!(instances.len(), 1);
        let refused = prepared
            .attach_home_world(
                &phase,
                world.owner.publication_port(),
                world.owner.branch_port(),
                world.owner.observation_port(),
                world.owner.inspection_port(),
                world.owner.recovery_port(),
                root,
                |_, _| None,
                Arc::new(MaintenanceMustRemainUnentered),
            )
            .err()
            .expect("a memory root cannot issue QueryReadmitted for a Home");
        let HomeWorldAttachmentCustody::Prepared {
            _prepared: prepared,
            cause,
            ..
        } = refused.custody
        else {
            panic!("refusal must retain the original preparation before assembly")
        };
        assert!(
            cause
                .to_string()
                .contains("exact installed Native/Bridge basis"),
            "must reach the actual World check: {cause}"
        );
        assert_eq!(
            prepared
                .bootstrap
                .managed_indexes
                .configured_instances_for_test(),
            instances
        );
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(prepared);
        assert_eq!(
            drops.load(Ordering::SeqCst),
            1,
            "the actual owner releases the same provider once"
        );
    });
}
