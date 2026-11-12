//! Genuine co-anchored birth, retirement, restoration, and inverse admission history.
mod publication;
use publication::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId},
    mvcc::*,
    runtime::*,
    schema::{register_current_publication_storage, RelationalSchemaRegistry},
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    publications::{begin_main_transaction, release_test_commit_snapshot},
    AspectSchemaFixture,
};

pub(super) struct Journey {
    pub(super) restored: CommitResult,
    pub(super) runtime: RelationalRuntime,
    pub(super) installed: RelationalInstalledApplicationPublicationAuthority,
    pub(super) retired: CommitResult,
    pub(super) source: EntityId,
}

pub(super) fn run(execution: &ExecutionRequest<'_, '_>) -> Journey {
    let history = InstalledHistory::new();
    let original = history.birth(execution);
    let retired = history.retire(execution, &original);
    let (restored, fresh) = history.restore(execution, &retired, &original);
    let retired = history.retire(execution, &fresh);
    Journey {
        restored,
        runtime: history.runtime,
        installed: history.installed,
        retired,
        source: fresh.sources[0],
    }
}

struct InstalledHistory {
    runtime: RelationalRuntime,
    owner: worth_proof::InstalledApplicationPublicationOwner,
    installed: RelationalInstalledApplicationPublicationAuthority,
}
struct Roster {
    sources: Vec<EntityId>,
    members: Vec<EntityId>,
    relations: Vec<RelationId>,
}
impl InstalledHistory {
    fn new() -> Self {
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
        Self {
            runtime,
            owner,
            installed,
        }
    }

    fn birth(&self, execution: &ExecutionRequest<'_, '_>) -> Roster {
        let sources = [
            target("authored-source"),
            target("co-anchor-a"),
            target("co-anchor-b"),
        ];
        let mut transaction = begin_main_transaction(&self.runtime);
        transaction
            .push_batch(creates(&sources), execution.allocation_policy())
            .unwrap();
        let born = transaction.commit(execution, &self.runtime).unwrap();
        let sources = sources
            .iter()
            .map(|t| born.created_entity(t).unwrap())
            .collect::<Vec<_>>();
        release_test_commit_snapshot(&self.runtime, &born);
        let mut roster = Roster {
            sources,
            members: Vec::new(),
            relations: Vec::new(),
        };
        for route in 0..3 {
            self.birth_outputs(execution, route, &mut roster);
        }
        roster.members.extend(&roster.sources);
        roster
    }

    fn birth_outputs(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        route: usize,
        roster: &mut Roster,
    ) {
        let source = roster.sources[route];
        let meaning = meaning(route, source);
        let targets = output_targets(route, 0, 2);
        let (batch, relations) = create_batch(source, &targets, route, 0, 2);
        let basis = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1;
        let absence = self
            .runtime
            .readmit_required_output_route_absence(execution, &self.installed, &basis, &meaning)
            .unwrap();
        let born = publish(
            &self.runtime,
            execution,
            &self.installed,
            batch,
            RequiredOutputPublication::Birth {
                admission: self.owner.admit_required_output("scale-realizer", [1; 32]),
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
        collect(
            &self.runtime,
            &born,
            &targets,
            &relations,
            &mut roster.members,
            &mut roster.relations,
        );
    }

    fn retire(&self, execution: &ExecutionRequest<'_, '_>, roster: &Roster) -> CommitResult {
        let roles = roster
            .members
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
        let basis = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1;
        let predecessor = self
            .runtime
            .readmit_authored_source_retirement_with_co_anchors(
                execution,
                &self.installed,
                &basis,
                roster.sources[0],
                &roster.sources[1..],
                &roles,
            )
            .unwrap()
            .unwrap();
        let admission = self
            .owner
            .admit_authored_output_retirements(predecessor.authorities())
            .unwrap();
        let mut batch = WorkerIntentBatch::new("retire-source");
        for &relation_id in &roster.relations {
            batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent { relation_id },
            )));
        }
        for &entity_id in &roster.members {
            batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id },
            )));
        }
        let retired = publish(
            &self.runtime,
            execution,
            &self.installed,
            batch,
            RequiredOutputPublication::AuthoredCascadeRetirement {
                admission,
                predecessor,
            },
        );
        release_test_commit_snapshot(&self.runtime, &retired);
        retired
    }

    fn restore(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        retired: &CommitResult,
        original: &Roster,
    ) -> (CommitResult, Roster) {
        let fresh = [
            target("restored-source"),
            target("restored-co-a"),
            target("restored-co-b"),
        ];
        let basis = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1;
        let co_anchors = original.sources[1..]
            .iter()
            .copied()
            .zip(fresh[1..].iter().cloned())
            .collect::<Vec<_>>();
        let restoration = self
            .runtime
            .prepare_authored_source_restoration_with_co_anchors(
                execution,
                &self.installed,
                &basis,
                retired.commit.commit_id,
                (original.sources[0], fresh[0].clone()),
                &co_anchors,
            )
            .unwrap()
            .unwrap();
        let admission = self
            .owner
            .admit_authored_output_retirements(restoration.authorities())
            .unwrap();
        let batch = restoration
            .extend_batch(execution, creates(&fresh))
            .unwrap();
        let restored = publish(
            &self.runtime,
            execution,
            &self.installed,
            batch,
            RequiredOutputPublication::AuthoredCascadeRestoration {
                admission,
                restoration,
            },
        );
        let sources = fresh
            .iter()
            .map(|t| restored.created_entity(t).unwrap())
            .collect::<Vec<_>>();
        assert_ne!(sources[0], original.sources[0]);
        release_test_commit_snapshot(&self.runtime, &restored);
        let event = serde_json::from_str::<serde_json::Value>(
            &serde_json::to_string(restored.publication().envelope.as_ref()).unwrap(),
        )
        .unwrap();
        let event = &event["authored_cascade_restoration"];
        let members = event["entities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| serde_json::from_value(pair[1].clone()).unwrap())
            .collect();
        let relations = event["relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pair| serde_json::from_value(pair[1].clone()).unwrap())
            .collect();
        (
            restored,
            Roster {
                sources,
                members,
                relations,
            },
        )
    }
}
