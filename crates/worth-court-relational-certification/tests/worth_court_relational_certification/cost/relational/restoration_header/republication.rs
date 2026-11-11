//! Actual writer propagation reaches other restored routes' incident relations.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{mvcc::*, runtime::RelationalRuntime, transactions::*};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{release_test_commit_snapshot, string_aspect_field_patch},
};

pub(super) fn complete(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    restored: &CommitResult,
) {
    for prior in restored.required_output_publications(installed, execution) {
        let prior = prior.unwrap();
        let meaning = prior.meaning().clone();
        let roles = prior
            .members()
            .map(|(role, entity_name, _, id)| RequiredOutputRoleTarget {
                role: role.to_owned(),
                entity_name: entity_name.to_owned(),
                kind: worth_relational::facade::identity::KindId(1),
                posture: RequiredOutputRolePosture::Preserve,
                target: EntityReference::Existing(id),
            })
            .collect::<Vec<_>>();
        let EntityReference::Existing(entity_id) = roles[0].target else {
            unreachable!()
        };
        let batch =
            WorkerIntentBatch::new("propagate-restored-output").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: string_aspect_field_patch([(
                        aspect_key("name"),
                        field_key("name"),
                        "record",
                    )]),
                }),
            ));
        let admission = owner
            .admit_required_output_succession(
                owner.admit_required_output("scale-realizer", [3; 32]),
                prior.authority(),
            )
            .unwrap();
        let propagated = super::journey::publish(
            runtime,
            execution,
            installed,
            batch,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: prior,
                meaning,
                roles,
            },
        );
        release_test_commit_snapshot(runtime, &propagated);
    }
}
