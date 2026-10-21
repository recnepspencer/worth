//! The currentness allowance decides reuse against a real native fact.

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::LocatorAuthority;

use super::marked_facts_permit_reuse;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

#[test]
fn a_marked_fact_is_reverified_at_the_real_native_snapshot() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let entity = world
                .selected_product(&phase.test_host_request())
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            let graph = world.application.runtime.primary_graph().unwrap();
            let status_ref = AccountStatus::reference();
            let status = graph
                .layout()
                .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
                .unwrap()
                .clone();
            let locator = AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                status.aspect().aspect_key().clone(),
                status.field_path().clone(),
            );

            world
                .application
                .primary_provider
                .graph
                .with_runtime(&execution, |runtime| {
                    let basis = runtime
                        .admit_branch_basis(&runtime.main_branch_identity())
                        .unwrap();
                    let snapshot = runtime
                        .snapshots()
                        .snapshot_for_observation(&basis.observation())
                        .unwrap();
                    let revision = runtime
                        .read_truth()
                        .project_snapshot(&snapshot)
                        .unwrap()
                        .entity_field_revision(&execution, entity, &locator)
                        .expect("fixture read request remains live")
                        .unwrap();
                    let facts = [WorthQueryApplicationObservedFact::SourceFieldRevision {
                        entity_id: entity,
                        locator: locator.clone(),
                        native_revision: Some(revision),
                    }];

                    // An ample allowance proves the marked fact current, so any `false`
                    // below comes from exhaustion alone, never from a changed source.

                    assert!(matches!(
                        marked_facts_permit_reuse(&execution, &facts, runtime, &snapshot),
                        Ok(true)
                    ));

                    runtime.snapshots().release_snapshot(&snapshot).unwrap();
                });
        })
        .expect("the fixture borrows its installed host phase");
}

#[test]
fn a_clean_settlement_reuses_without_fact_reverification() {
    use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
    use crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary;
    use crate::domain_computation::primary_graph::application_attempt::PreparedDecisionReuseContext;
    use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputPosture;
    use crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole;
    use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
    use crate::domain_computation::primary_graph::application_contribution::PriorAbsence;
    use crate::domain_computation::primary_graph::application_contribution::WorthQueryDecisionContextDependencies;
    use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerInputReuseContract;
    use crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceSelection;
    use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
    use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistration;
    use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
    use crate::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
    use crate::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutputMutable;
    use crate::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation;
    use crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence;
    use crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey;
    use crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness;
    use crate::domain_computation::primary_graph::output_lineage::ProductCoordinate;
    use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
    use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
    use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputCorrespondence;
    use crate::domain_computation::primary_graph::DecisionContextUse;
    use crate::equality_type_id::EqualityTypeId as TypeId;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.borrowed_request_for_test();
            let selected_product = world.selected_product(&phase.test_host_request());
            let entity = selected_product
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            let (_, product, _) = selected_product.into_parts();
            let observation = product.observation();
            let graph = world.application.runtime.primary_graph().unwrap();
            let handle = graph.integration_handle();
            let owner = &handle.source_owner.invalidation_owner;
            let status_ref = AccountStatus::reference();
            let status = graph
                .layout()
                .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
                .unwrap();
            let locator = AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                status.aspect().aspect_key().clone(),
                status.field_path().clone(),
            );
            let correspondence = Arc::new(
                WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
                    TypeId::of::<()>(),
                    TypeId::of::<()>(),
                    std::iter::empty(),
                    &[WorthQueryCheckpointOutputRole {
                        role: "account".to_owned(),
                        posture: WorthQueryApplicationOutputPosture::Preserve,
                        entity_name: "Account".to_owned(),
                        entity,
                    }],
                    |_| Some(TypeId::of::<()>()),
                    &execution,
                )
                .unwrap(),
            );
            let source = SemanticSource {
                runtime_authority: world.application.runtime.authority_identity().as_u64(),
                schema: world.application.installed_schema.binding_identity(),
                scope: WorthQueryOperationScopeEntityBinding::from_entity(entity),
                output_binding: OutputBindingIdentity::declared("ProbeOutput"),
            };
            let identity = RecordedSettlementIdentity::retain(
                &source,
                ProductCoordinate {
                    occurrence: observation.lifecycle_incarnation(),
                    generation: observation.reference_generation().get(),
                },
                0,
            );
            let selection = WorthQueryObservedSourceSelection::selecting_for_test(1);
            let input_key = || {
                PreparedInputReuseKey::new(
                    selection.clone(),
                    [0x61; 32],
                    InstalledProducerEdition::for_test([0x51; 32]),
                )
            };
            let context = || {
                PreparedDecisionReuseContext::new(
                    WorthQueryProducerInputReuseContract::canonical_bitwise(
                        WorthQueryDecisionContextDependencies::NONE,
                    ),
                    None,
                    None,
                    None,
                )
                .unwrap()
            };

            handle.with_runtime(&execution, |runtime| {
                let basis = runtime
                    .admit_branch_basis(&runtime.main_branch_identity())
                    .unwrap();
                let snapshot = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
                let revision = runtime
                    .read_truth()
                    .project_snapshot(&snapshot)
                    .unwrap()
                    .entity_field_revision(&execution, entity, &locator)
                    .expect("fixture read request remains live")
                    .unwrap();
                let facts: Arc<[_]> =
                    Arc::from([WorthQueryApplicationObservedFact::SourceFieldRevision {
                        entity_id: entity,
                        locator: locator.clone(),
                        native_revision: Some(revision),
                    }]);

                let witness = PreparedNativeOutputWitness::prepare_republication(
                    &correspondence,
                    graph.layout(),
                )
                .unwrap()
                .expect("the installed Account admits its native witness")
                .finish(&correspondence, runtime, &snapshot)
                .unwrap();
                let evidence = ComputationSourceEvidence::for_test(false);
                owner
                    .register_settlement(SettlementRegistration {
                        computation: None,
                        work_membership: None,
                        identity: Arc::clone(&identity),
                        facts: evidence.retain_facts(Arc::clone(&facts)),
                        output_facts: None,
                        read_basis: selected.clone(),
                        stale_at_read_basis: Default::default(),
                        requirement: None,
                        upstream: Default::default(),
                    })
                    .unwrap();
                assert!(
                    matches!(
                        owner.currentness(&selected, &identity),
                        Ok(SourceSettlementCurrentness::Clean)
                    ),
                    "the native settlement must start Clean"
                );
                assert!(
                    matches!(
                        marked_facts_permit_reuse(&execution, facts.iter(), runtime, &snapshot),
                        Ok(true)
                    ),
                    "the native probe confirms the original handler fact"
                );

                let boundary = CompletedHandlerFactBoundary::completed_for_test(1);
                let completed = boundary
                    .seal_decision_reuse(context(), DecisionContextUse::default())
                    .unwrap();
                let row = RecordedOutput {
                    required_publication: None,
                    computation_source: evidence,
                    native_prior_checkpoint: None,
                    performed_origin: None,
                    consumed_outputs: Arc::from([]),
                    completed_handler_facts: Some(boundary),
                    completed_decision_reuse: Some(completed),
                    prepared_input_reuse_key: Some(input_key()),
                    native_output_witness: OnceLock::from(witness),
                    settlement_identity: Arc::clone(&identity),
                    correspondence: Arc::clone(&correspondence),
                    source_identity: None,
                    source_partition_identity: None,
                    producer_dependency_identity: None,
                    idempotency_key_identity: [0x41; 32],
                    mutable: Mutex::new(RecordedOutputMutable::new(
                        None,
                        Some(evidence.retain_facts(facts)),
                        RecordedComputation::Absent(PriorAbsence::FirstRun),
                    )),
                };
                let candidate = super::RetainedInputCutoffCandidate::from_exact_cell(Arc::new(
                    OnceLock::from(row),
                ));

                let decision = candidate
                    .eligible_for_reuse(
                        &execution,
                        &input_key(),
                        &context(),
                        None,
                        runtime,
                        &snapshot,
                        &selected,
                        owner,
                    )
                    .unwrap();
                assert!(
                    decision.is_some(),
                    "a Clean settlement must reuse without loading its facts"
                );
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
            });
        })
        .expect("the fixture borrows its installed host phase");
}
