//! Five-field transport/native publication, NOT workflow approval authorization.
//! The separate public court supplies the genuine workflow/checkpoint journey.
use super::super::super::fixture::{
    prepare_relational_mutation_on_application, release_test_commit_snapshot, AccountIdentity,
    AccountLabel, AccountMembershipTag, AccountStatus,
};
use super::*;
use crate::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as Publication;
use crate::domain_computation::primary_graph::{
    workflow::{workflow_approval_fields, WorkflowApprovalFieldText, WorkflowApprovalMeaning},
    WorthQueryApplicationAttemptDenialKind as AttemptKind, WorthQueryApplicationCommittedChanges,
};
use std::{collections::BTreeMap, num::NonZeroUsize};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    facade::{AspectFieldLocator, AspectValue, InternedString},
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::PartitionId,
    symbols::{ClientKey, Symbol},
    transactions::{
        AspectFieldPatch, CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent,
        WorkerIntentBatch,
    },
};

fn payer(cancellation: &CancellationSource) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 2 * 1024 * 1024, 1_000_000),
        ),
        deadline: None,
        cancellation: cancellation.token(),
    })
    .unwrap()
}

// Only the five actual admitted encoder outputs and native principal/grant/scope
// are relevant. Other labels are descriptive fixture data, not a workflow hash,
// definition, RequiredWorkflowApproval or guarded permission.
fn meaning(admission: &Admission, execution: &ExecutionRequest<'_, '_>) -> WorkflowApprovalMeaning {
    let authority = admission
        .workflow_approval_authority()
        .expect("genuine capability authority");
    WorkflowApprovalMeaning {
        identity: authority.decision_identity,
        identity_text: "five-field-native-transport".to_owned(),
        instance_identity: "transport-only".to_owned(),
        definition_content_identity: "not-a-workflow-definition".to_owned(),
        program_revision: "transport-only".to_owned(),
        proposal: admission.scope_entity_id(), evidence: Box::default(),
        approval_operation: "transport-only".to_owned(),
        target_operation: "transport-only".to_owned(),
        installed_capability_identity: "transport-only".to_owned(),
        approver: authority.principal, grant: authority.grant,
        authorization_decision: authority.decision_identity,
        authorization_request: admission.workflow_approval_request_snapshot(execution).unwrap().expect("request encoded"),
        authorization_principal: admission.workflow_approval_principal_snapshot(execution).unwrap().expect("principal encoded"),
        capability_authority_identity: authority.capability_authority_identity,
        authorization_lineage: admission.workflow_approval_lineage_snapshot(execution).unwrap().expect("lineage encoded"),
        authorization_support: admission.workflow_approval_support_snapshot(execution).unwrap().expect("support encoded"),
        authorization_dependencies: admission.workflow_approval_dependencies_snapshot(execution).unwrap().expect("dependencies encoded"),
        action: format!("{:?}", authority.action), purpose: format!("{:?}", authority.purpose),
        timeline: authority.timeline.canonical_name(), sampled_value: format!("{:?}", authority.sampled_value),
        expiry: authority.expiry, scope: admission.scope_entity_id(),
        decision: crate::domain_computation::primary_graph::application_attempt::WorkflowApprovalDecision::Approve,
    }
}
fn originals(meaning: &WorkflowApprovalMeaning) -> [String; 5] {
    [
        &meaning.authorization_request,
        &meaning.authorization_principal,
        &meaning.authorization_lineage,
        &meaning.authorization_support,
        &meaning.authorization_dependencies,
    ]
    .map(|text| text.as_str().to_owned())
}
fn locators(
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
) -> [&AspectFieldLocator; 5] {
    [
        &layout.approval.authorization_request,
        &layout.approval.authorization_principal,
        &layout.approval.authorization_lineage,
        &layout.approval.authorization_support,
        &layout.approval.authorization_dependencies,
    ]
}

#[test]
fn five_admitted_envelopes_publish_original_json_and_keep_payer_until_last_native_pin() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let cancellation = CancellationSource::new();
    let payer = payer(&cancellation);
    let baseline = payer.charged_memory_bytes();
    assert_eq!(baseline, 0);
    let request = live_scope();
    let (pin, symbols, expected) = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let principal = authenticated_principal(&host, &world, &request);
            let admission = admitted_capability_operation(&host, &world, &principal, &request);
            let execution = ExecutionRequest::leased(&payer);
            let meaning = meaning(&admission, &execution);
            let expected = originals(&meaning); // Oracle copies precede native admission.
            let selected = world.selected_product(&host);
            let (_, product, _) = selected.into_parts();
            let basis = product.relational_basis();
            let graph = world.application.runtime.primary_graph().unwrap();
            let handle = graph.integration_handle();
            let retained_layout = graph.layout();
            let layout = retained_layout.workflow();
            let admitted_text = handle.with_runtime(&execution, |runtime| {
                WorkflowApprovalFieldText::admit(runtime, &execution, basis, &meaning).unwrap()
            });
            let all_fields =
                workflow_approval_fields(layout, &meaning, basis, &admitted_text).unwrap();
            let locators = locators(layout);
            // Account is the existing valid native fixture carrier. These are
            // transport labels, never incomplete platform approval records.
            let identity = AccountIdentity::reference();
            let status = AccountStatus::reference();
            let membership = AccountMembershipTag::reference();
            let label = AccountLabel::reference();
            let field = |entity, aspect, name| {
                retained_layout
                    .field_locator(entity, aspect, name)
                    .unwrap()
                    .clone()
            };
            let identity_locator = field(identity.entity(), identity.aspect(), identity.field());
            let status_locator = field(status.entity(), status.aspect(), status.field());
            let membership_locator =
                field(membership.entity(), membership.aspect(), membership.field());
            let label_locator = field(label.entity(), label.aspect(), label.field());
            let account_kind = retained_layout.entity_kind(label.entity()).unwrap();
            let text = |value: &str| AspectValue::String(InternedString::Raw(value.to_owned()));
            let mut carriers = Vec::with_capacity(5);
            let mut batch = WorkerIntentBatch::new("five-envelope-transport");
            for (index, source_locator) in locators.iter().enumerate() {
                let key = format!("five-envelope-transport-{index}");
                let fields = BTreeMap::from([
                    (identity_locator.clone(), text(&key)),
                    (status_locator.clone(), text("active")),
                    (membership_locator.clone(), text("open")),
                    (
                        label_locator.clone(),
                        all_fields.get(*source_locator).unwrap().clone(),
                    ),
                ]);
                let created = CreatedEntityRef {
                    partition_id: PartitionId::main(),
                    kind_id: account_kind,
                    client_key: ClientKey::raw(key),
                };
                batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: created.partition_id,
                    kind_id: created.kind_id,
                    client_key: created.client_key.clone(),
                    fields: AspectFieldPatch::from(fields),
                })));
                carriers.push(created);
            }
            let prepared = prepare_relational_mutation_on_application(
                &execution,
                &world.application,
                batch,
                &request,
            );
            let Ok(Publication::Performed(performed)) = prepared.execute(execution) else {
                panic!("the real native/World transport publication must perform");
            };
            let commit = performed
                .component_results()
                .relational_commit_result()
                .unwrap();
            let changes = WorthQueryApplicationCommittedChanges::from_commit(commit);
            let mut actual_entities = std::collections::BTreeSet::new();
            let symbols = carriers
                .iter()
                .map(|created| {
                    let entity = commit
                        .created_entity(created)
                        .expect("actual Account carrier created reference");
                    assert!(
                        actual_entities.insert(entity),
                        "five distinct actual carrier records"
                    );
                    let actual = changes
                        .committed_field_values(entity, &[&label_locator])
                        .expect("actual immutable committed AccountLabel");
                    let [AspectValue::String(InternedString::Symbol(symbol))] = actual.as_slice()
                    else {
                        panic!("actual committed label must be exactly one Symbol: {actual:?}");
                    };
                    Symbol(symbol.0)
                })
                .collect::<Vec<_>>();
            assert_eq!(symbols.len(), 5);
            let published_basis = performed.commit().basis().relational_basis();
            let pin = handle.with_runtime_mut(&execution, |runtime| {
                handle
                    .ensure_primary_indexes_for_basis(&execution, runtime, published_basis)
                    .unwrap();
                let pin = runtime
                    .read_truth()
                    .pin_read_capability(&commit.snapshot, &[], &execution)
                    .unwrap();
                assert_eq!(pin.snapshot(), &commit.snapshot);
                for (symbol, expected) in symbols.iter().zip(&expected) {
                    assert_eq!(
                        pin.symbols().resolve(*symbol).unwrap().as_bytes(),
                        expected.as_bytes()
                    );
                }
                release_test_commit_snapshot(runtime, commit);
                pin
            });
            drop(changes);
            drop(performed.consume());
            // Everything except native pin and test-oracle strings is local:
            // Meaning, five handles, actual field vectors, admission and request end.
            (pin, symbols, expected)
        })
        .expect("genuine installed fixture host scope");
    assert!(payer.charged_memory_bytes() > baseline);
    drop(request);
    drop(world); // Includes actual World/application/runtime/configuration owners.
    assert!(
        payer.charged_memory_bytes() > baseline,
        "only the real native pin retains payload custody"
    );
    for (symbol, expected) in symbols.iter().zip(&expected) {
        assert_eq!(
            pin.symbols().resolve(*symbol).unwrap().as_bytes(),
            expected.as_bytes()
        );
        assert!(pin
            .symbols()
            .retained_text(*symbol)
            .unwrap()
            .charged_payload_bytes()
            .is_some());
    }
    drop(pin);
    // Equal-text interning may consolidate values; never require five charges
    // or per-Symbol refunds. No independently owned payload remains here.
    assert_eq!(payer.charged_memory_bytes(), baseline);
}

#[test]
fn original_cancellation_reaches_query_native_receiving_before_any_dictionary_or_field_publication()
{
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let cancellation = CancellationSource::new();
    let payer = payer(&cancellation);
    let request = live_scope();
    world.application.with_host_advancement(|phase| {
        let host = phase.test_host_request();
        let principal = authenticated_principal(&host, &world, &request);
        let admission = admitted_capability_operation(&host, &world, &principal, &request);
        let execution = ExecutionRequest::leased(&payer);
        let meaning = meaning(&admission, &execution);
        let expected = originals(&meaning);
        assert!(expected.iter().all(|text| !text.is_empty()), "ALL five original encoders succeeded before cancellation");
        let charge = payer.charged_memory_bytes();
        assert!(charge > 0);
        let selected = world.selected_product(&host);
        let (_, product, _) = selected.into_parts();
        let basis = product.relational_basis();
        let observer = host.borrowed_request_for_test();
        let handle = world.application.runtime.primary_graph().unwrap().integration_handle();
        // Open the real receiver while live. Cancellation occurs inside, with
        // no intervening outer with_runtime check or new callback/test control.
        handle.with_runtime_mut(&execution, |runtime| {
            assert!(runtime.owner_component_services().basis_port().compare_current_exact(basis).is_ok());
            let before = runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap();
            let before_pin = runtime.read_truth().pin_read_capability(&before, &[], &observer).unwrap();
            cancellation.cancel();
            let denied = match WorkflowApprovalFieldText::admit(runtime, &execution, basis, &meaning) {
                Err(denied) => denied,
                Ok(_) => panic!("native receive must preserve original cancellation"),
            };
            assert_eq!(denied.kind(), AttemptKind::Cancelled);
            assert_eq!(denied.relational_execution_cause(), Some(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled));
            assert!(runtime.owner_component_services().basis_port().compare_current_exact(basis).is_ok(), "original carried basis still names the current head");
            let after = runtime.snapshots().snapshot_for_observation(&basis.observation()).unwrap();
            let after_pin = runtime.read_truth().pin_read_capability(&after, &[], &observer).unwrap();
            assert_eq!(after.version_id(), before.version_id(), "no field/head publication");
            assert_eq!(after_pin.symbols(), before_pin.symbols(), "no dictionary publication");
            assert_eq!(originals(&meaning), expected, "original five buffers unchanged");
            assert_eq!(payer.charged_memory_bytes(), charge, "refusal acquired no payload custody");
            drop(before_pin);
            drop(after_pin);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &after);
            // Legal live counterpart uses the same already-paid original bytes;
            // it does not reset the cancelled request or manufacture admission.
            let admitted_text = WorkflowApprovalFieldText::admit(runtime, &observer, basis, &meaning).unwrap();
            let fields = workflow_approval_fields(
                world.application.runtime.primary_graph().unwrap().layout().workflow(),
                &meaning, basis, &admitted_text,
            ).unwrap();
            assert!(locators(world.application.runtime.primary_graph().unwrap().layout().workflow()).iter().all(|locator| {
                matches!(fields.get(*locator), Some(AspectValue::String(InternedString::Symbol(_))))
            }));
        });
    }).expect("genuine installed fixture host scope");
}
