//! A pending upstream cannot hide a conclusive change to a candidate's own output.
use super::*;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryCheckpointOutputRole,
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    output_lineage::{SealedNativeOutputWitness, WorthQueryApplicationOutputCorrespondence},
    WorthQueryApplicationOutputPosture,
};

#[test]
fn pending_candidate_rejects_changed_own_evidence_without_certifying_upstream() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = layout
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = layout
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let a = RecordedSettlementIdentity::retain(
        &source(OutputBindingIdentity::declared("StatusOutput")),
        coordinate,
        0,
    );
    let b = RecordedSettlementIdentity::retain(
        &source(OutputBindingIdentity::declared("DownstreamOutput")),
        coordinate,
        0,
    );
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole {
            role: "account".into(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Account".into(),
            entity,
        }],
        |_| Some(TypeId::of::<()>()),
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
    )
    .unwrap();
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); handle.with_runtime_mut(&host_custody.borrowed_request_for_test(), |runtime| {
        write_field(&host_custody.borrowed_request_for_test(), runtime, entity, label.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let a_facts: Arc<[_]> =
            Arc::from([field_fact(&host_custody.borrowed_request_for_test(), runtime, &before_handle, entity, status.clone())]);
        let own_fact = field_fact(&host_custody.borrowed_request_for_test(), runtime, &before_handle, entity, label.clone());
        register(
            owner,
            Arc::clone(&a),
            Arc::clone(&a_facts),
            &before,
            OrdSet::new(),
        );
        register(
            owner,
            Arc::clone(&b),
            Arc::from([own_fact.clone()]),
            &before,
            OrdSet::unit(Arc::clone(&a)),
        );
        let a_evidence = ConsumedOutputEvidence::retained_for_test(Arc::clone(&a), a_facts, vec![], None, Arc::new(before.clone()));
        let b_evidence = ConsumedOutputEvidence::retained_for_test(Arc::clone(&b), Arc::from([own_fact]), vec![a_evidence], None, Arc::new(before.clone()));
        // The original native output expectations are sealed from authentic
        // installed aspects, never from the later output head.
        let truth = runtime.read_truth();
        let mut output_facts = vec![WorthQueryApplicationObservedFact::Entity {
            entity_id: entity,
            kind: truth
                .exact_snapshot_live_entity_kind(&before_handle, entity)
                .unwrap(),
        }];
        for aspect in layout.native_output_aspects("Account") {
            output_facts.push(WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id: entity,
                aspect: aspect.clone(),
                native_revision: truth
                    .exact_snapshot_entity_aspect_version(&before_handle, entity, aspect)
                    .unwrap(),
            });
        }
let witness = SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &output_facts)
        .unwrap()
        .unwrap();


        write_field(&host_custody.borrowed_request_for_test(), runtime, entity, status, "closed");
        let (pending_handle, pending) = snapshot(runtime);
        assert!(matches!(
            owner
                .currentness(&pending, &b)
                .unwrap(),
            SourceSettlementCurrentness::PendingUpstream(_)
        ));

        assert_eq!(
            ConsumedOutputEvidence::verify_many_with_admission(&host_custody.borrowed_request_for_test(),
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &pending_handle,
                &pending),
            Err(ConsumedOutputVerificationStop::PendingUpstream),
            "unchanged own facts cannot certify pending upstream"
        );

        assert_eq!(
            ConsumedOutputEvidence::verify_at_observation(&host_custody.borrowed_request_for_test(),
                &b,
                &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::from([])).for_comparison().unwrap(),
                &[],
                None,
                &witness,
                owner,
                runtime,
                &pending_handle,
                &pending),
            Ok(ConsumedOutputVerification::ChangedUpstream),
            "the original output witness conclusively rejects the changed output"
        );
        write_field(&host_custody.borrowed_request_for_test(), runtime, entity, label, "changed-own-field");
        let (changed_handle, changed) = snapshot(runtime);

        assert_eq!(
            ConsumedOutputEvidence::verify_many_with_admission(&host_custody.borrowed_request_for_test(),
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &changed_handle,
                &changed),
            Ok(ConsumedOutputVerification::ChangedDirectFact(0))
        );
        for snapshot in [before_handle, pending_handle, changed_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    }) }).expect("the fixture read owns its installed host call");
}
