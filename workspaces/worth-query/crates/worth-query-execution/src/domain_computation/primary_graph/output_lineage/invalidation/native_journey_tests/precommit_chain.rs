//! A selected B output carries its actual A edge into C's precommit check.
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use im::OrdSet;

use super::{field_fact, register, snapshot, AccountLabel, AccountStatus};
use super::{DownstreamOutput, SemanticSource, StatusOutput};
use crate::domain_computation::primary_graph::{
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    output_lineage::{
        invalidation::{
            FullVerificationReason, InvalidationEditAdmission, SettlementRegistration,
            SourceSettlementCurrentness,
        },
        ProductCoordinate, RecordedSettlementIdentity,
    },
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation, AuthorizationWorld,
        TouchAccountOperation,
    },
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrincipalResolutionMode,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;

#[path = "precommit_chain/request_custody.rs"]
mod request_custody;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[test]
fn provider_precommit_refuses_earlier_three_hop_evidence_at_current_submission() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout()
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
    let a =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<StatusOutput>()), coordinate, 0);
    let b = RecordedSettlementIdentity::retain(
        &source(TypeId::of::<DownstreamOutput>()),
        coordinate,
        0,
    );
    publish_field(&world, entity, label.clone(), "prime");
    let (before_handle, before, a_facts) = handle.with_runtime(|runtime| {
        let (selected, basis) = snapshot(runtime);
        let facts: Arc<[_]> = Arc::from([field_fact(runtime, &selected, entity, status.clone())]);
        (selected, basis, facts)
    });
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
        Arc::from([]),
        &before,
        OrdSet::unit(Arc::clone(&a)),
    );
    let a_evidence = ConsumedOutputEvidence::retained_for_test(
        owner,
        Arc::clone(&a),
        a_facts,
        Vec::new(),
        None,
        Arc::new(before.clone()),
    );
    let b_evidence = ConsumedOutputEvidence::retained_for_test(
        owner,
        Arc::clone(&b),
        Arc::from([]),
        vec![a_evidence],
        None,
        Arc::new(before),
    );
    request_custody::assert_backing_paid_by_the_request(owner, std::slice::from_ref(&b_evidence));

    // World publication advances both the native and product selected roots.
    // The unrelated field leaves the actual consumed source fact current.
    publish_field(&world, entity, label, "unrelated-label");
    let (unrelated_handle, unrelated) = handle.with_runtime(snapshot);
    let mut work = owner.edit_admission();
    assert_eq!(
        handle.with_runtime(
            |runtime| ConsumedOutputEvidence::verify_many_with_admission(
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &unrelated_handle,
                &unrelated,
                &mut work,
            )
        ),
        Ok(ConsumedOutputVerification::Current),
    );

    // Two closures on one admitted meter must consume two real sets of Work
    // and scratch. A fresh allowance per closure would incorrectly pass the
    // second call under either one-short budget.
    let verify_unrelated = |admission: &mut InvalidationEditAdmission| {
        handle.with_runtime(|runtime| {
            ConsumedOutputEvidence::verify_many_with_admission(
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &unrelated_handle,
                &unrelated,
                admission,
            )
        })
    };
    let mut shared = owner.edit_admission();
    let first = verify_unrelated(&mut shared);
    assert_eq!(first, Ok(ConsumedOutputVerification::Current));
    let first_work = shared.charged_work();
    let first_bytes = shared.charged_bytes();
    let second = verify_unrelated(&mut shared);
    assert_eq!(second, Ok(ConsumedOutputVerification::Current));
    let total_work = shared.charged_work();
    let total_bytes = shared.charged_bytes();
    assert!(total_work > first_work && total_bytes > first_bytes);
    request_custody::assert_carried_native_verification(owner, total_work, verify_unrelated);
    request_custody::assert_closures_share_the_meter(total_work, total_bytes, verify_unrelated);

    // C has already consumed B. Changing A leaves B pending at the new
    // product-selected source, before C's provider commit can prepare effects.
    publish_field(&world, entity, status, "closed");
    let (after_handle, after) = handle.with_runtime(snapshot);
    let mut work = owner.edit_admission();
    assert_eq!(
        handle.with_runtime(
            |runtime| ConsumedOutputEvidence::verify_many_with_admission(
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &after_handle,
                &after,
                &mut work,
            )
        ),
        Err(ConsumedOutputVerificationStop::PendingUpstream),
    );
    assert!(
        work.charged_work() > 0,
        "the wrapper debits the work spent before denial"
    );

    // A discontinuous B row has no direct facts. Its shared A evidence must
    // still catch the changed source fact after full verification is required.
    owner
        .register_settlement(
            SettlementRegistration {
                work_membership: None,
                identity: Arc::clone(&b),
                facts: RetainedSourceFacts::for_test(false, Arc::from([])),
                output_facts: None,
                read_basis: after.clone(),
                stale_at_read_basis: OrdSet::new(),
                requirement: Some(FullVerificationReason::DeclaredChangeUnavailable),
                upstream: OrdSet::unit(Arc::clone(&a)),
            },
            &mut owner.edit_admission(),
        )
        .expect("discontinuous B row is retained at the selected source");
    assert!(matches!(
        owner.currentness(&after, &b, &mut owner.edit_admission()),
        Ok(SourceSettlementCurrentness::FullVerificationRequired(
            FullVerificationReason::DeclaredChangeUnavailable
        ))
    ));
    let mut work = owner.edit_admission();
    assert_eq!(
        handle.with_runtime(
            |runtime| ConsumedOutputEvidence::verify_many_with_admission(
                std::slice::from_ref(&b_evidence),
                owner,
                runtime,
                &after_handle,
                &after,
                &mut work,
            )
        ),
        Ok(ConsumedOutputVerification::ChangedUpstream),
    );
    handle.with_runtime_mut(|runtime| {
        for selected in [before_handle, unrelated_handle, after_handle] {
            runtime.snapshots().release_snapshot(&selected).unwrap();
        }
    });

    // The real provider commit entry must consume the earlier B evidence
    // before preparing any effect. C is submitted at the current native and
    // Product basis, so a historical-basis CAS cannot explain its refusal.
    let admitted_program = |replacement: &str| {
        let request = live_scope();
        let external = world.authenticate("alice", Duration::from_secs(60), &request);
        let selected = world.selected_product();
        let principal = selected
            .resolve_authenticated_principal(
                &world.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let account = selected
            .resolve_entity(
                AccountStatus::reference(),
                "closed".to_owned(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let operation = world
            .application
            .installed_schema()
            .installed_operation(TouchAccountOperation::reference())
            .unwrap();
        let admission = selected
            .authorize_operation(
                &principal,
                &account,
                &operation,
                Default::default(),
                &request,
            )
            .unwrap();
        let (_, projection, _) = world
            .invariant
            .project_admitted_operation(&admission, |reader, account| {
                reader
                    .require_decision_field(account, AccountStatus::reference())
                    .unwrap();
            })
            .unwrap()
            .into_parts();
        let reads = world
            .application
            .begin_projected_application_read_attempt(admission, projection)
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies()
            .unwrap()
            .begin_effect_program();
        let account = effects.existing_entity(&account).unwrap();
        effects
            .write_field(&account, AccountStatus::reference(), replacement.to_owned())
            .unwrap();
        effects.finish().unwrap()
    };
    let stale = admitted_program("must-not-publish").with_consumed_output_for_test(b_evidence);
    let product_before = world.selected_product().product().observation().clone();
    let (head_before_handle, head_before) = handle.with_runtime(snapshot);
    let denied = world.application.compare_and_commit_application(
        stale,
        WorthQueryApplicationIdempotencyBinding::new([211; 32], [212; 32]),
    );
    let WorthQueryApplicationCommitOutcome::Denied(denial) = denied else {
        panic!("the provider must refuse changed upstream evidence before effects: {denied:?}");
    };
    assert_eq!(
        denial.detail(),
        Some("consumed output changed before application publication"),
        "precommit refusal kind {:?}, stage {:?}",
        denial.kind(),
        denial.stage(),
    );
    let (head_after_handle, head_after) = handle.with_runtime(snapshot);
    assert_eq!(
        head_after, head_before,
        "the denied C commit cannot move native truth"
    );
    assert_eq!(
        world.selected_product().product().observation(),
        &product_before,
        "the denied C commit cannot move Product truth"
    );
    handle.with_runtime_mut(|runtime| {
        runtime
            .snapshots()
            .release_snapshot(&head_before_handle)
            .unwrap();
        runtime
            .snapshots()
            .release_snapshot(&head_after_handle)
            .unwrap();
    });
    assert!(matches!(
        world.application.compare_and_commit_application(
            admitted_program("fresh-valid-commit"),
            WorthQueryApplicationIdempotencyBinding::new([213; 32], [214; 32]),
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}

fn publish_field(
    world: &AuthorizationWorld,
    entity: EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let batch =
        WorkerIntentBatch::new("native-world-current-output-journey").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    AspectValue::String(InternedString::Raw(value.to_owned())),
                )])),
            }),
        ));
    publish_relational_mutation(world, batch);
}
