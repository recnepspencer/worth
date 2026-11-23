//! A public Native create/grow/retire/restore journey with independent route width.
use super::fixtures::{create_batch, creates, meaning, output_targets, role, target};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    mvcc::*,
    runtime::*,
    schema::{register_current_publication_storage, RelationalSchemaRegistry},
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    publications::{begin_main_transaction, release_test_commit_snapshot},
    AspectSchemaFixture,
};

pub(super) struct Journey {
    pub(super) restored: CommitResult,
    pub(super) phase_work: [u64; 4],
    pub(super) restored_relations: usize,
}

fn measure_phase<T>(
    execution: &ExecutionRequest<'_, '_>,
    operation: impl FnOnce() -> T,
) -> (T, u64) {
    let (value, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| operation())
        .expect("unbounded phase work measurement completes");
    (value, report.charged_work())
}

pub(super) fn run(
    execution: &ExecutionRequest<'_, '_>,
    publications: usize,
    members: usize,
    edges: usize,
) -> Journey {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let registry = register_current_publication_storage(
        fixture.build_registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed =
        installed.with_source_scope_owned_producers([("scale-realizer".to_owned(), KindId(1))]);
    let ((source_id, mut all_members, mut all_edges, priors), created_work) =
        measure_phase(execution, || {
            let source = target("authored-source");
            let mut transaction = begin_main_transaction(&runtime);
            transaction
                .push_batch(
                    creates(std::slice::from_ref(&source)),
                    execution.allocation_policy(),
                )
                .unwrap();
            let born = transaction.commit(execution, &runtime).unwrap();
            let source_id = born.created_entity(&source).unwrap();
            release_test_commit_snapshot(&runtime, &born);
            let mut all_members = Vec::new();
            let mut all_edges = Vec::new();
            let mut priors = Vec::new();
            let mut route_source = source_id;
            for route in 0..publications {
                let meaning = meaning(route, route_source);
                let targets = output_targets(route, 0, members);
                let (batch, relations) = create_batch(route_source, &targets, route, 0, edges);
                let (_, basis) = runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap();
                let absence = runtime
                    .readmit_required_output_route_absence(execution, &installed, &basis, &meaning)
                    .unwrap();
                let born = publish(
                    &runtime,
                    execution,
                    &installed,
                    batch,
                    RequiredOutputPublication::Birth {
                        admission: owner.admit_required_output("scale-realizer", [1; 32]),
                        absence,
                        meaning,
                        roles: targets
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                role(
                                    i,
                                    EntityReference::Created(t.clone()),
                                    RequiredOutputRolePosture::Create,
                                )
                            })
                            .collect(),
                    },
                );
                let ids = collect(
                    &runtime,
                    &born,
                    &targets,
                    &relations,
                    &mut all_members,
                    &mut all_edges,
                );
                let next_source = ids[0];
                priors.push((
                    born.required_output_publication(&installed).unwrap(),
                    ids,
                    route_source,
                ));
                route_source = next_source;
            }
            (source_id, all_members, all_edges, priors)
        });
    let ((), grown_work) = measure_phase(execution, || {
        for (route, (prior, ids, source)) in priors.into_iter().enumerate() {
            let targets = output_targets(route, 1, members);
            let (batch, relations) = create_batch(source, &targets, route, 1, edges);
            let mut roles = ids
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    role(
                        i,
                        EntityReference::Existing(*id),
                        RequiredOutputRolePosture::Preserve,
                    )
                })
                .collect::<Vec<_>>();
            roles.extend(targets.iter().enumerate().map(|(i, t)| {
                role(
                    i + ids.len(),
                    EntityReference::Created(t.clone()),
                    RequiredOutputRolePosture::Create,
                )
            }));
            let admission = owner
                .admit_required_output_succession(
                    owner.admit_required_output("scale-realizer", [2; 32]),
                    prior.authority(),
                )
                .unwrap();
            let grown = publish(
                &runtime,
                execution,
                &installed,
                batch,
                RequiredOutputPublication::Preserve {
                    admission,
                    predecessor: prior,
                    meaning: meaning(route, source),
                    roles,
                },
            );
            collect(
                &runtime,
                &grown,
                &targets,
                &relations,
                &mut all_members,
                &mut all_edges,
            );
        }
    });
    let restored_relations = all_edges.len();
    let (retired, retirement_work) = measure_phase(execution, || {
        all_members.push(source_id);
        let roles = all_members
            .iter()
            .enumerate()
            .map(|(i, id)| {
                role(
                    i,
                    EntityReference::Existing(*id),
                    RequiredOutputRolePosture::Retire,
                )
            })
            .collect::<Vec<_>>();
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let predecessor = runtime
            .readmit_authored_source_retirement(execution, &installed, &basis, source_id, &roles)
            .unwrap()
            .unwrap();
        let admission = owner
            .admit_authored_output_retirements(predecessor.authorities())
            .unwrap();
        let mut batch = WorkerIntentBatch::new("retire-source");
        for relation_id in all_edges {
            batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent { relation_id },
            )));
        }
        for entity_id in all_members {
            batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id },
            )));
        }
        let retired = publish(
            &runtime,
            execution,
            &installed,
            batch,
            RequiredOutputPublication::AuthoredCascadeRetirement {
                admission,
                predecessor,
            },
        );
        release_test_commit_snapshot(&runtime, &retired);
        retired
    });
    let ((restored, fresh_source), restoration_work) = measure_phase(execution, || {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let fresh_source = target("restored-source");
        let restoration = runtime
            .prepare_authored_source_restoration(
                execution,
                &installed,
                &basis,
                retired.commit.commit_id,
                source_id,
                fresh_source.clone(),
            )
            .unwrap()
            .unwrap();
        let admission = owner
            .admit_authored_output_retirements(restoration.authorities())
            .unwrap();
        let batch = restoration
            .extend_batch(execution, creates(std::slice::from_ref(&fresh_source)))
            .unwrap();
        let restored = publish(
            &runtime,
            execution,
            &installed,
            batch,
            RequiredOutputPublication::AuthoredCascadeRestoration {
                admission,
                restoration,
            },
        );
        super::republication::complete(&runtime, execution, &installed, &owner, &restored);
        (restored, fresh_source)
    });
    assert_ne!(restored.created_entity(&fresh_source).unwrap(), source_id);
    release_test_commit_snapshot(&runtime, &restored);
    Journey {
        restored,
        phase_work: [created_work, grown_work, retirement_work, restoration_work],
        restored_relations,
    }
}

fn collect(
    runtime: &RelationalRuntime,
    result: &CommitResult,
    targets: &[CreatedEntityRef],
    relations: &[CreatedRelationRef],
    members: &mut Vec<EntityId>,
    edges: &mut Vec<worth_relational::facade::identity::RelationId>,
) -> Vec<EntityId> {
    let ids = targets
        .iter()
        .map(|t| result.created_entity(t).unwrap())
        .collect::<Vec<_>>();
    members.extend(&ids);
    edges.extend(
        relations
            .iter()
            .map(|r| result.created_relation(r).unwrap()),
    );
    release_test_commit_snapshot(runtime, result);
    ids
}
pub(super) fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(installed, publication)
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("public journey must publish")
    };
    runtime.settle_performed_publication(performed).unwrap()
}
