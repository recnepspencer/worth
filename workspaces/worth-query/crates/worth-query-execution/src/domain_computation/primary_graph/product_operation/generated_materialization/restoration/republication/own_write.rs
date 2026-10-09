//! Native suspension/restoration must preserve the computation's original basis.
use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::own_write_fixture::with_generated_own_write,
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            live_scope,
            own_write_computation::OwnWriteOutputs,
            retained_output_capacity::{ReadRetainedAccount, RetainedFamily},
            AccountIdentity, AccountLabel, AccountMembershipTag, AccountStatus,
        },
    },
    WorthQueryCurrentOutputDenialKind,
};
use std::{any::TypeId, collections::BTreeMap};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::{
    branch::RelationalEntityMaterialization, transactions::AspectFieldPatch,
};
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

#[test]
fn republication_cannot_certify_output_computed_before_its_own_write() {
    suspend_restore_and_read(false);
}

#[test]
fn restoration_fallback_cannot_certify_output_computed_before_its_own_write() {
    suspend_restore_and_read(true);
}

fn suspend_restore_and_read(exhaust_registration: bool) {
    with_generated_own_write(|world, receipt, _, resources| {
        let application = &world.application;
        let request = live_scope();
        let graph = application.runtime.primary_graph().unwrap();
        let handle = graph.integration_handle();
        let selected = world.selected_product();
        let (_, product, basis) = selected.into_parts();
        let observation = product.observation();
        let exact = handle.output_lineage.lock().unwrap().qualified_output::<OwnWriteOutputs>(
            application.runtime.authority_identity().as_u64(),
            &application.installed_schema.binding_identity(), receipt.principal_scope().scope(),
            observation.lifecycle_incarnation(), observation.reference_generation().get(),
            crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new([11;32]),
            crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([11;32]),
        ).expect("the real committed record qualifies its suspension");
        // Provider registration is irrelevant to record continuity; all runtime,
        // correspondence, facts and source identity come from that exact record.
        let producer = ProducerQualification {

            binding_type: TypeId::of::<OwnWriteOutputs>(),
            output_binding_type: TypeId::of::<OwnWriteOutputs>(),
            binding_identity: "test-own-write", provider_identity: "test-own-write",
            runtime_source_identity: crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity::new([11;32]),
            checkpoint_source_identity: crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([11;32]),
            recorded_source_identity: exact.source_identity,
            source_partition_identity: exact.source_partition_identity,
            producer_dependency_identity: exact.producer_dependency_identity,
            idempotency_key_identity: exact.idempotency_key_identity,
            runtime_authority: exact.runtime_authority, schema: exact.schema, scope: exact.scope,
            output_occurrence: observation.lifecycle_incarnation(),
            output_generation: observation.reference_generation().get(),
            observed_source_facts: exact.observed_source_facts, resources: exact.resources,
        };
        let generated = receipt
            .output_correspondence()
            .created_entity_ids()
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), 1);
        let generated = generated[0];
        let kind = handle.with_runtime(|runtime| {
            runtime
                .read_truth()
                .exact_snapshot_live_entity_kind(basis.snapshot_handle(), generated)
                .unwrap()
        });
        let layout = graph.layout();
        let label = AccountLabel::reference();
        let status = AccountStatus::reference();
        let identity = AccountIdentity::reference();
        let tag = AccountMembershipTag::reference();
        let fields = [
            (
                layout
                    .field_locator(label.entity(), label.aspect(), label.field())
                    .unwrap()
                    .clone(),
                "computed-from-1",
            ),
            (
                layout
                    .field_locator(status.entity(), status.aspect(), status.field())
                    .unwrap()
                    .clone(),
                "generated",
            ),
            (
                layout
                    .field_locator(identity.entity(), identity.aspect(), identity.field())
                    .unwrap()
                    .clone(),
                "computed-own-write",
            ),
            (
                layout
                    .field_locator(tag.entity(), tag.aspect(), tag.field())
                    .unwrap()
                    .clone(),
                "open",
            ),
        ]
        .into_iter()
        .map(|(locator, value)| {
            (
                locator,
                AspectValue::String(InternedString::Raw(value.to_owned())),
            )
        })
        .collect::<BTreeMap<_, _>>();
        let prepared = handle.with_runtime(|runtime| {
            runtime
                .owner_component_services()
                .materialization_port()
                .prepare_generated_materialization_suspension(
                    product.relational_basis(),
                    &[generated],
                )
                .unwrap()
        });
        let (candidate, completion) = prepared.into_parts();
        let published = product
            .publication_binding()
            .prepare_relational_candidate(candidate, &request, true)
            .unwrap()
            .execute();
        let RuntimeWorldPublicationOutcome::Performed(performed) = published else {
            panic!("suspension publishes");
        };
        let commit = performed
            .component_results()
            .relational_commit_result()
            .unwrap()
            .clone();
        drop(performed.consume());
        let suspended = completion.complete(commit).unwrap();
        drop(basis);
        drop(product);
        let (_, product, basis) = world.selected_product().into_parts();
        let prepared = handle.with_runtime(|runtime| {
            runtime
                .owner_component_services()
                .materialization_port()
                .prepare_generated_rematerialization(
                    product.relational_basis(),
                    suspended.custody,
                    vec![RelationalEntityMaterialization {
                        entity_id: generated,
                        kind_id: kind,
                        fields: AspectFieldPatch::from(fields),
                    }],
                    vec![],
                )
                .unwrap()
        });
        let (candidate, completion, _) = prepared.into_parts();
        let published = product
            .publication_binding()
            .prepare_relational_candidate(candidate, &request, true)
            .unwrap()
            .execute();
        let RuntimeWorldPublicationOutcome::Performed(performed) = published else {
            panic!("restoration publishes");
        };
        let commit = performed
            .component_results()
            .relational_commit_result()
            .unwrap()
            .clone();
        let observation = performed.consume().take_successor_observation().unwrap();
        completion.complete(commit).unwrap();
        // Another reservation may exhaust the derived ledger after World has
        // published but before the lineage recorder runs. The fallback must
        // carry the same qualification evidence as performed republication.
        let held = exhaust_registration.then(|| {
            resources
                .reserve_retained_capacity(
                    resources.installation().maximum_retained_bytes
                        - resources.retained_capacity_bytes(),
                )
                .unwrap()
        });
        application.record_restored_generated_output(&observation, exact.correspondence, producer);
        drop(held);
        drop(basis);
        drop(product);
        handle
            .output_lineage
            .lock()
            .unwrap()
            .install_output_families(BTreeMap::from([(
                "test.retained-account-capacity".to_owned(),
                vec![(
                    TypeId::of::<OwnWriteOutputs>(),
                    "computed-account".to_owned(),
                )],
            )]));
        let (_, restored_product, _) = world.selected_product().into_parts();
        let observation = restored_product.observation();
        let family = handle
            .output_lineage
            .lock()
            .unwrap()
            .resolve_current_family(
                application.runtime.authority_identity().as_u64(),
                &application.installed_schema.binding_identity(),
                receipt.principal_scope().scope(),
                "test.retained-account-capacity",
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                usize::MAX,
            )
            .unwrap();
        assert_eq!(family.candidates.len(), 1);
        let candidate = &family.candidates[0];
        assert_eq!(candidate.verification_requirement, exhaust_registration.then_some(FullVerificationReason::CheckpointRestore),
            "the tests must reach the performed republication and its capacity fallback respectively");
        drop(restored_product);
        let principal = authenticated_principal(world, &request);
        let account = resolved_account(world, "2", &request);
        let operation = application
            .installed_schema()
            .installed_operation(ReadRetainedAccount::reference())
            .unwrap();
        let admitted = world
            .selected_product()
            .authorize_operation(
                &principal,
                &account,
                &operation,
                TypedMutationPreconditions::new(),
                &request,
            )
            .unwrap();
        crate::domain_computation::primary_graph::output_lineage::own_write_fixture::evict_index_with_unrelated_write(world, &resources);
        let answer = world
            .invariant
            .project_admitted_operation(
                &admitted,
                |reader, root| {
                    reader
                        .current_output::<RetainedFamily, _>(root)
                        .map(|_| ())
                        .map_err(|denial| denial.kind())
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .into_parts()
            .0;
        assert_eq!(
            answer,
            Err(WorthQueryCurrentOutputDenialKind::StaleSource),
            "restored output still contains the computation from x=1, while x=2"
        );
        assert!(
            candidate.observed_source_facts.for_comparison().is_none(),
            "deriving a restored record must retain the actual own-write evidence"
        );
    });
}
