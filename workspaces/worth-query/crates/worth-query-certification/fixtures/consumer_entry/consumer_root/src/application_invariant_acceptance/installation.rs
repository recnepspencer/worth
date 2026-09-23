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

pub(super) fn assert_program_cannot_omit_an_installed_rule() {
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
    let result = installation::in_memory_program(
        crate::application_program::validated_omitted_installed_rule_program()
            .expect("the omitted-rule program is declaration-valid"),
        ConsumerSchema::declaration().expect("the contributed declaration is valid"),
        configuration,
        limits,
        |graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
                .expect("the contributed principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        },
    );
    let denial = match result {
        Err(installation::WorthQueryInMemoryApplicationDenial::Program(denial)) => denial,
        Err(other) => panic!("expected omitted-rule denial, received {other}"),
        Ok(_) => panic!("a program that omits an installed rule was accepted"),
    };
    assert_eq!(
        denial.subject(),
        "undeclared installed rule: PositiveParameterCount"
    );
}

pub(super) fn assert_program_cannot_omit_a_required_binding() {
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
    let result = installation::in_memory_program(
        crate::application_program::validated_omitted_program_binding(),
        ConsumerSchema::declaration().expect("the contributed declaration is valid"),
        configuration,
        limits,
        |graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
                .expect("the contributed principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        },
    );
    let denial = match result {
        Err(installation::WorthQueryInMemoryApplicationDenial::Program(denial)) => denial,
        Err(other) => panic!("expected omitted-binding denial, received {other}"),
        Ok(_) => panic!("a program that omits a required installed binding was accepted"),
    };
    assert_eq!(
        denial.subject(),
        "undeclared program binding: worth.query.certification.planar-source-adjustment.v1"
    );
}

pub(super) fn assert_required_output_source_cannot_be_an_action() {
    use worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring;

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
    let program = ApplicationProgramAuthoring::<
        ConsumerSchema,
        crate::application_program::RequiredSourceAsActionProgram,
    >::begin()
    .validated_program()
    .expect("the overlapping action is declaration-valid");
    let result = installation::in_memory_program(
        program,
        ConsumerSchema::declaration().unwrap(),
        configuration,
        limits,
        |graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<ConsumerSchema>())
                .expect("the contributed principal mapping is installed");
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64,
                authentication::external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
        },
    );
    let application = result.expect("the overlapping program remains declaration-valid");
    let denial = match application.admit_program_operation::<
        worth_query_topology_entry::AdjustPlanarSource,
    >() {
        Err(denial) => denial,
        Ok(_) => panic!("required output source was admitted as an ordinary action"),
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired,
    );
}

pub(super) fn assert_repeated_optional_member_correspondence(world: &ConsumerWorld) {
    use crate::application_program::correspondence::{
        OptionalAdjustmentCorrespondence, OptionalAdjustmentRow,
    };
    use worth_query_consumer_values::{PlanarOperation, PositiveLength};
    use worth_query_host::facade::declaration::application_program::ApplicationOptionalMemberEdit;
    use worth_query_topology_entry::{EditPlanar, PlanarEditBinding};

    let correspondence = world
        .application
        .installed_program()
        .repeated_optional_member::<
            PlanarEditBinding<ConsumerSchema>,
            OptionalAdjustmentCorrespondence,
        >()
        .expect("the program installs its declared row correspondence");
    let first = correspondence.initialize(
        &OptionalAdjustmentRow {
            body_key: "row-a".to_owned(),
            replacement_y: Some(PositiveLength::new(2).unwrap()),
        },
        "source-a".to_owned(),
    );
    assert_eq!(first.target(), "row-a");
    assert_eq!(first.initial_member().map(PositiveLength::get), Some(2));
    assert!(first
        .apply(ApplicationOptionalMemberEdit::Unchanged)
        .is_none());

    let set = first
        .apply(ApplicationOptionalMemberEdit::Set(
            PositiveLength::new(3).unwrap(),
        ))
        .expect("an edit generates an action");
    assert_eq!(set.required_source(), "source-a");
    assert_eq!(set.input().scope_key, "row-a");
    let PlanarOperation::Adjust(adjustments) = &set.input().operation else {
        panic!("the correspondence generated the wrong operation")
    };
    assert_eq!(adjustments[0].body_key, "row-a");
    assert_eq!(PositiveLength::get(&adjustments[0].replacement_y), 3);

    let clear = first
        .apply(ApplicationOptionalMemberEdit::Clear)
        .expect("an explicit clear generates an action");
    let PlanarOperation::Adjust(adjustments) = &clear.input().operation else {
        panic!("the correspondence generated the wrong clear operation")
    };
    assert!(adjustments.is_empty());

    let second = correspondence.initialize(
        &OptionalAdjustmentRow {
            body_key: "row-b".to_owned(),
            replacement_y: None,
        },
        "source-b".to_owned(),
    );
    assert_eq!(second.target(), "row-b");
    assert_eq!(second.initial_member(), None);
    assert_eq!(second.required_source(), "source-b");

    let external = world
        .application
        .installed_program()
        .external_input_provider::<
            EditPlanar,
            crate::application_program::external_input::NeutralExternalProvider,
        >()
        .expect("the neutral external provider slot is installed on the action");
    let provider = crate::application_program::external_input::NeutralExternalProvider::new(7);
    let changed = external.resolve(&provider, "material").unwrap();
    assert_eq!(changed.resolution().provenance(), &"neutral-catalog");
    provider.change(8);
    assert_eq!(
        changed.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Changed)
    );
    let removed = external.resolve(&provider, "material").unwrap();
    provider.remove();
    assert_eq!(
        removed.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Removed)
    );
    provider.change(9);
    let invalid = external.resolve(&provider, "material").unwrap();
    provider.invalidate();
    assert_eq!(
        invalid.admit(&provider).err(),
        Some(crate::application_program::external_input::NeutralExternalDenial::Invalid)
    );
    provider.change(10);
    let admitted = external
        .resolve(&provider, "material")
        .unwrap()
        .admit(&provider)
        .unwrap()
        .into_resolution();
    assert_eq!(admitted.values(), &10);
    assert_eq!(admitted.revision(), &10);
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
