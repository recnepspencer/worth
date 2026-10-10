//! The currentness allowance decides reuse against a real native fact.

use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};
use worth_relational::facade::mvcc::CompanionPreflightBudget;

use super::{marked_facts_permit_reuse, InvalidationEditAdmission};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    tests::fixture::{installed_authorization_world, live_scope, AccountStatus},
    WorthQueryPrincipalResolutionMode,
};

fn currentness(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 1024 * 1024,
    })
}

#[test]
fn exhausted_currentness_while_reverifying_a_marked_fact_selects_fresh_without_stopping() {
    let world = installed_authorization_world(true);
    let entity = world
        .selected_product()
        .resolve_entity(
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
        .with_runtime(|runtime| {
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
                .entity_field_revision(entity, &locator)
                .unwrap();
            let facts = [WorthQueryApplicationObservedFact::SourceFieldRevision {
                entity_id: entity,
                locator: locator.clone(),
                native_revision: Some(revision),
            }];

            // An ample allowance proves the marked fact current, so any `false`
            // below comes from exhaustion alone, never from a changed source.
            let mut ample = currentness(1 << 20);
            let full = ample.remaining_work();
            assert!(matches!(
                marked_facts_permit_reuse(&facts, runtime, &snapshot, &mut ample),
                Ok(true)
            ));
            let required = u64::try_from(full - ample.remaining_work()).unwrap();
            assert!(required > 1, "re-verification costs a visit plus its probe");

            // Every smaller allowance runs out inside re-verification. Each one
            // selects Fresh and leaves the demand running instead of returning
            // a terminal stop.
            for work in 1..required {
                assert!(
                    matches!(
                        marked_facts_permit_reuse(
                            &facts,
                            runtime,
                            &snapshot,
                            &mut currentness(work)
                        ),
                        Ok(false)
                    ),
                    "currentness work {work} of {required} must select Fresh"
                );
            }
            assert!(matches!(
                marked_facts_permit_reuse(&facts, runtime, &snapshot, &mut currentness(required)),
                Ok(true)
            ));
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        });
}

#[test]
fn a_clean_settlement_reuses_with_one_unit_and_no_fact_load() {
    use crate::domain_computation::{
        authorization::WorthQueryOperationScopeEntityBinding,
        primary_graph::{
            application_attempt::{
                CompletedHandlerFactBoundary, PreparedDecisionReuseContext,
                WorthQueryApplicationOutputPosture, WorthQueryCheckpointOutputRole,
            },
            application_contribution::{
                InstalledProducerEdition, PriorAbsence, WorthQueryDecisionContextDependencies,
                WorthQueryProducerInputReuseContract,
            },
            application_query::WorthQueryObservedSourceSelection,
            output_binding_identity::OutputBindingIdentity,
            output_lineage::{
                invalidation::{SettlementRegistration, SourceSettlementCurrentness},
                recorded_output::{RecordedOutput, RecordedOutputMutable},
                retained_computation::RecordedComputation,
                ComputationSourceEvidence, PreparedInputReuseKey, PreparedNativeOutputWitness,
                ProductCoordinate, RecordedSettlementIdentity, SemanticSource,
                WorthQueryApplicationOutputCorrespondence,
            },
            DecisionContextUse,
        },
    };
    use std::{
        any::TypeId,
        sync::{Arc, Mutex, OnceLock},
    };

    let world = installed_authorization_world(true);
    let selected_product = world.selected_product();
    let entity = selected_product
        .resolve_entity(
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
            Default::default(),
            vec![WorthQueryCheckpointOutputRole {
                role: "account".to_owned(),
                posture: WorthQueryApplicationOutputPosture::Preserve,
                entity_name: "Account".to_owned(),
                entity,
            }],
            |_| Some(TypeId::of::<()>()),
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

    handle.with_runtime(|runtime| {
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
            .entity_field_revision(entity, &locator)
            .unwrap();
        let facts: Arc<[_]> = Arc::from([WorthQueryApplicationObservedFact::SourceFieldRevision {
            entity_id: entity,
            locator: locator.clone(),
            native_revision: Some(revision),
        }]);
        let mut admission = currentness(1 << 20);
        let witness = PreparedNativeOutputWitness::prepare_republication(
            &correspondence,
            graph.layout(),
            owner,
            &mut admission,
        )
        .unwrap()
        .expect("the installed Account admits its native witness")
        .finish(&correspondence, runtime, &snapshot)
        .unwrap();
        let evidence = ComputationSourceEvidence::for_test(false);
        owner
            .register_settlement(
                SettlementRegistration {
                    work_membership: None,
                    identity: Arc::clone(&identity),
                    facts: evidence.retain_facts(Arc::clone(&facts)),
                    output_facts: None,
                    read_basis: selected.clone(),
                    stale_at_read_basis: Default::default(),
                    requirement: None,
                    upstream: Default::default(),
                },
                &mut admission,
            )
            .unwrap();
        assert!(
            matches!(
                owner.currentness(&selected, &identity, &mut admission),
                Ok(SourceSettlementCurrentness::Clean)
            ),
            "the native settlement must start Clean"
        );
        assert!(
            matches!(
                marked_facts_permit_reuse(facts.iter(), runtime, &snapshot, &mut currentness(1)),
                Ok(false)
            ),
            "one unit cannot load the native handler fact"
        );

        let boundary = CompletedHandlerFactBoundary::completed_for_test(1);
        let completed = boundary
            .seal_decision_reuse(context(), DecisionContextUse::default())
            .unwrap();
        let row = RecordedOutput {
            computation_source: evidence,
            native_prior_checkpoint: None,
            _retained_capacity: None,
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
                None,
                RecordedComputation::Absent(PriorAbsence::FirstRun),
            )),
        };
        let candidate =
            super::RetainedInputCutoffCandidate::from_exact_cell(Arc::new(OnceLock::from(row)));
        let mut allowance = currentness(1);
        let decision = candidate
            .eligible_for_reuse(
                &input_key(),
                &context(),
                None,
                runtime,
                &snapshot,
                &selected,
                owner,
                &mut admission,
                &mut allowance,
            )
            .unwrap();
        assert!(
            decision.is_some(),
            "a Clean settlement must reuse without loading its facts"
        );
        assert_eq!(
            allowance.remaining_work(),
            1,
            "Clean reuse must spend zero currentness units"
        );
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
