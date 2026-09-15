use super::{authentication, resources, seed};
use crate::{ConsumerProgram, ConsumerSchema};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    application_installation::{self as installation, WorthQueryInMemoryApplicationLimits},
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    domain, primary_graph, runtime,
};
use worth_query_topology_entry::{ConsumerPrincipalBinding, TopologyConfiguration};

#[path = "installation/unavailable_program.rs"]
mod unavailable_program;
pub(super) use unavailable_program::{assert_rule_provider_postures, install_unavailable};

pub(super) struct ConsumerWorld {
    pub(super) application:
        installation::WorthQueryProgramApplicationRuntime<ConsumerSchema, ConsumerProgram>,
    pub(super) invariant_calls: Arc<AtomicUsize>,
    pub(super) invariant_probe: Arc<AtomicUsize>,
    pub(super) producer_authorization_denials: Arc<AtomicUsize>,
}

pub(super) fn install(
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) -> ConsumerWorld {
    install_with_candidate_bytes(foreign, 8192)
}

pub(super) fn assert_plain_installation_requires_program() {
    let configuration = (
        TopologyConfiguration {
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        },
        Arc::new(AtomicUsize::new(0)),
    );
    let limits = WorthQueryInMemoryApplicationLimits::new(
        resources::world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(4096, 8192, 4096).unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(4096, 4096, 4096, 32).unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    );
    let result = installation::in_memory::<ConsumerSchema>(
        ConsumerSchema::declaration().expect("the contributed declaration is valid"),
        configuration,
        limits,
        |_, _| Ok(()),
    );
    let Err(installation::WorthQueryInMemoryApplicationDenial::ApplicationProgramRequired(binding)) =
        result
    else {
        panic!("plain installation must reject a program-required source binding")
    };
    assert_eq!(
        binding,
        "worth.query.certification.planar-source-adjustment.v1"
    );
}

pub(super) fn install_with_candidate_bytes(
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
    candidate_bytes: u64,
) -> ConsumerWorld {
    install_with_resource_bytes(foreign, candidate_bytes, 4096)
}

pub(super) fn install_with_query_bytes(
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
    query_bytes: usize,
) -> ConsumerWorld {
    install_with_resource_bytes(foreign, 8192, query_bytes)
}

fn install_with_resource_bytes(
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
    candidate_bytes: u64,
    query_bytes: usize,
) -> ConsumerWorld {
    let topology_calls = Arc::new(AtomicUsize::new(0));
    let parameter_calls = Arc::new(AtomicUsize::new(0));
    let invariant_calls = Arc::new(AtomicUsize::new(0));
    let invariant_probe = Arc::new(AtomicUsize::new(0));
    let producer_authorization_denials = Arc::new(AtomicUsize::new(0));
    let configuration = (
        TopologyConfiguration {
            setup_calls: Arc::clone(&topology_calls),
            invariant_calls: Arc::clone(&invariant_calls),
            invariant_probe: Arc::clone(&invariant_probe),
            producer_authorization_denials: Arc::clone(&producer_authorization_denials),
        },
        Arc::clone(&parameter_calls),
    );
    let limits = WorthQueryInMemoryApplicationLimits::new(
        resources::world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
            4096,
            candidate_bytes,
            4096,
        )
        .unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(4096, query_bytes, 4096, 32)
            .unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    );
    let application = installation::in_memory_program::<ConsumerSchema, ConsumerProgram>(
        crate::application_program::validated_program()
            .expect("the application program is complete"),
        ConsumerSchema::declaration().expect("the contributed declaration is valid"),
        configuration,
        limits,
        |graph, installed| {
            reject_foreign_invariant_factory(installed, foreign);
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
                .expect("the contributed principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            seed::seed_cycles(graph);
            Ok(())
        },
    )
    .expect("the contributed configuration and initial state publish one application");
    assert_eq!(topology_calls.load(Ordering::SeqCst), 1);
    assert_eq!(parameter_calls.load(Ordering::SeqCst), 1);
    ConsumerWorld {
        application,
        invariant_calls,
        invariant_probe,
        producer_authorization_denials,
    }
}
fn reject_foreign_invariant_factory(
    installed: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
    foreign: &domain::WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) {
    use worth_query_decl::facade::application_schema::{
        ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
    };
    use worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantFactories;
    use worth_query_topology_entry::{PositivePlanarTurn, PositiveTurnRule};

    let foreign_invariant = foreign
        .installed_invariant(
            PositivePlanarTurn::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )
        .expect("the independent installation has the same invariant meaning");
    let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(installed);
    let denial = factories
        .bind(
            &foreign_invariant,
            |_| -> Result<PositiveTurnRule<ConsumerSchema>, String> {
                panic!("a foreign installed handle must never invoke its factory")
            },
        )
        .expect_err("equivalent invariant meaning does not transfer installation authority");
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ForeignInvariantFactory,
    );
}
