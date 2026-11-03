//! Owned publications cannot escape through legacy retirement or producer succession.
use super::*;
use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;

fn basis(
    runtime: &RelationalRuntime,
) -> worth_relational::facade::branch::AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}
fn ids(roles: &[RequiredOutputRoleTarget]) -> Vec<EntityId> {
    roles
        .iter()
        .map(|role| match role.target {
            EntityReference::Existing(id) => id,
            _ => unreachable!(),
        })
        .collect()
}
pub(super) fn refused(
    error: TransactionCommitError,
    owner: EntityId,
    publication: worth_relational::facade::history::CommitId,
    unavailable: bool,
) {
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("typed custody refusal: {error:?}")
    };
    let cause = error.authored_retirement_refusal().unwrap();
    assert!(
        matches!(cause, Cause::OwnedPublicationRequiresCascade { owner: actual, publication: id } if actual == owner && id == publication)
            && !unavailable
            || matches!(cause, Cause::PublicationOwnerUnavailable { owner: actual, publication: id } if actual == owner && id == publication)
                && unavailable,
        "{error:?}"
    );
    assert_eq!(error.context.affected_records.len(), 2);
    assert_eq!(error.context.affected_records[1], RecordRef::Entity(owner));
}
pub(super) fn prepare_refusal(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> TransactionCommitError {
    let before = basis(runtime);
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(installed, publication)
        .unwrap();
    let error = match runtime.prepare_validated_proposal(execution, proposal) {
        Ok(_) => panic!("owned publication escaped through a single-publication candidate"),
        Err(error) => error,
    };
    assert_eq!(basis(runtime).descriptor(), before.descriptor());
    error
}
#[allow(clippy::too_many_arguments)]
pub(super) fn check(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    roster: &[RequiredOutputRoleTarget],
    meanings: &[RequiredOutputPublicationMeaning],
    publications: &[worth_relational::facade::history::CommitId],
) {
    let before = basis(runtime);
    let all = ids(roster);
    for (index, meaning) in meanings.iter().enumerate() {
        let start = 1 + index * 2;
        let selected = &roster[start..start + 2];
        // Both a complete old-path roster and an anchorless partial roster refuse.
        for roles in [selected, &selected[..1], &selected[1..]] {
            refused(
                runtime
                    .readmit_authored_output_retirement(execution, installed, &before, roles)
                    .unwrap_err(),
                meaning.source_scope,
                publications[index],
                false,
            );
        }
        for role in selected {
            let id = ids(std::slice::from_ref(role))[0];
            let error = runtime
                .readmit_authored_source_retirement(
                    execution,
                    installed,
                    &before,
                    id,
                    std::slice::from_ref(role),
                )
                .unwrap_err();
            let TransactionCommitError::Publication { error, .. } = error else {
                panic!("{error:?}")
            };
            assert_eq!(
                error.authored_retirement_refusal(),
                Some(Cause::InvalidAuthoredAnchor)
            );
            assert_eq!(error.context.affected_records, [RecordRef::Entity(id)]);
        }
        let producer = ["parent-realizer", "child-realizer", "leaf-realizer"][index];
        let mut original_roles = selected.to_vec();
        for (i, role) in original_roles.iter_mut().enumerate() {
            role.role = format!("output-{i}");
            role.posture = RequiredOutputRolePosture::Create;
        }
        let prior = runtime
            .readmit_required_output_publication(
                execution,
                installed,
                &before,
                producer,
                &[1; 32],
                meaning,
                &original_roles,
            )
            .unwrap();
        let admission = owner
            .admit_authored_output_retirement(prior.authority())
            .unwrap();
        let batch = ids(selected).into_iter().fold(
            WorkerIntentBatch::new("warm-retire-bypass"),
            |batch, entity_id| {
                batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent { entity_id },
                )))
            },
        );
        refused(
            prepare_refusal(
                execution,
                runtime,
                installed,
                batch,
                RequiredOutputPublication::AuthoredRetirement {
                    admission,
                    predecessor: prior,
                    roles: selected.to_vec(),
                },
            ),
            meaning.source_scope,
            publications[index],
            false,
        );
        let prior = runtime
            .readmit_required_output_publication(
                execution,
                installed,
                &before,
                producer,
                &[1; 32],
                meaning,
                &original_roles,
            )
            .unwrap();
        let admission = owner
            .admit_required_output_succession(
                owner.admit_required_output(producer, [2; 32]),
                prior.authority(),
            )
            .unwrap();
        let batch = ids(selected).into_iter().fold(
            WorkerIntentBatch::new("producer-retire-all"),
            |batch, entity_id| {
                batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent { entity_id },
                )))
            },
        );
        let mut retiring = original_roles.clone();
        for role in &mut retiring {
            role.posture = RequiredOutputRolePosture::Retire;
        }
        refused(
            prepare_refusal(
                execution,
                runtime,
                installed,
                batch,
                RequiredOutputPublication::Preserve {
                    admission,
                    predecessor: prior,
                    meaning: meaning.clone(),
                    roles: retiring,
                },
            ),
            meaning.source_scope,
            publications[index],
            false,
        );
        if index + 1 < meanings.len() {
            // Producer replacement may retire a leaf, but may not sever the scope of a live child.
            let prior = runtime
                .readmit_required_output_publication(
                    execution,
                    installed,
                    &before,
                    producer,
                    &[1; 32],
                    meaning,
                    &original_roles,
                )
                .unwrap();
            let admission = owner
                .admit_required_output_succession(
                    owner.admit_required_output(producer, [2; 32]),
                    prior.authority(),
                )
                .unwrap();
            let target = targets(&["replacement-source"])[0].clone();
            let mut roles = original_roles;
            roles[0].posture = RequiredOutputRolePosture::Preserve;
            roles[1].posture = RequiredOutputRolePosture::Retire;
            roles.push(RequiredOutputRoleTarget {
                role: "replacement".into(),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(target.clone()),
                ..roles[0].clone()
            });
            let batch = creates("replace-source", &[target]).push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: all[start + 1],
                }),
            ));
            refused(
                prepare_refusal(
                    execution,
                    runtime,
                    installed,
                    batch,
                    RequiredOutputPublication::Preserve {
                        admission,
                        predecessor: prior,
                        meaning: meaning.clone(),
                        roles,
                    },
                ),
                all[start + 1],
                publications[index + 1],
                false,
            );
        }
    }
    // The reviewer's two-step sequence: rejecting the intermediate retirement must
    // leave its descendants live; then S + {Rp,Sc} is still incomplete, never an orphaning success.
    if meanings.len() == 3 {
        let error = runtime
            .readmit_authored_source_retirement(execution, installed, &before, all[0], &roster[..3])
            .unwrap_err();
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("{error:?}")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(Cause::MissingPublication)
        );
    }
    assert_eq!(basis(runtime).descriptor(), before.descriptor());
    let view = runtime
        .read_truth()
        .project_observation(&before.observation())
        .unwrap();
    for id in all {
        assert!(view
            .read_exact_entity_state(execution, id, |_, _| ())
            .unwrap()
            .is_some());
    }
}

pub(super) fn restore_without_owner(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    meaning: &RequiredOutputPublicationMeaning,
    publication: worth_relational::facade::history::CommitId,
) {
    let prior = runtime
        .readmit_authored_output_retirement_successor(
            execution,
            installed,
            &basis(runtime),
            meaning,
        )
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("parent-realizer", [2; 32]),
            prior.authority(),
        )
        .unwrap();
    let targets = targets(&["orphan-realization", "orphan-cell"]);
    let roles = targets
        .iter()
        .enumerate()
        .map(|(i, target)| RequiredOutputRoleTarget {
            role: format!("output-{i}"),
            entity_name: "node".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Create,
            target: EntityReference::Created(target.clone()),
        })
        .collect();
    let error = prepare_refusal(
        execution,
        runtime,
        installed,
        creates("orphan-restore", &targets),
        RequiredOutputPublication::Preserve {
            admission,
            predecessor: prior,
            meaning: meaning.clone(),
            roles,
        },
    );
    refused(error, meaning.source_scope, publication, true);
}
