//! Real installed source-owned publications through the public Native lifecycle.
use super::*;
use worth_relational::facade::{
    identity::*, mvcc::*, schema::*, symbols::ClientKey, transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
    AspectSchemaFixture,
};

pub(super) struct World {
    pub runtime: RelationalRuntime,
    pub owner: worth_proof::InstalledApplicationPublicationOwner,
    pub installed: RelationalInstalledApplicationPublicationAuthority,
    pub source: EntityId,
    pub source_commit: CommitId,
}
pub(super) fn runtime() -> RelationalRuntime {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let registry = register_current_publication_storage(
        fixture.build_registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap();
    worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap()
}
pub(super) fn target(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
    }
}
fn fields(value: &str) -> AspectFieldPatch {
    string_aspect_field_patch([(aspect_key("name"), field_key("name"), value)])
}
pub(super) fn create(key: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("authored-create").push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(key),
            fields: fields(key),
        },
    )))
}
fn roles(
    prior: &RelationalRequiredOutputPublication,
    posture: RequiredOutputRolePosture,
) -> Vec<RequiredOutputRoleTarget> {
    prior
        .members()
        .map(|(name, entity, _, id)| RequiredOutputRoleTarget {
            role: name.into(),
            entity_name: entity.into(),
            kind: KindId(1),
            posture,
            target: EntityReference::Existing(id),
        })
        .collect()
}
pub(super) fn commit(prior: &RelationalRequiredOutputPublication) -> CommitId {
    CommitId(prior.authority().publication_coordinate().0)
}
impl World {
    pub fn new(execution: &ExecutionRequest<'_, '_>) -> Self {
        Self::with_other_publication_owner(execution).0
    }
    pub fn with_other_publication_owner(
        execution: &ExecutionRequest<'_, '_>,
    ) -> (Self, RelationalInstalledApplicationPublicationAuthority) {
        let mut runtime = runtime();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(RelationalSchemaRegistry::new())
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let (_, other) = receipt.issue_application_publication_owner();
        let installed =
            installed.with_source_scope_owned_producers([("source-owned".into(), KindId(1))]);
        let mut tx = begin_main_transaction(&runtime);
        tx.push_batch(create("old-input"), execution.allocation_policy())
            .unwrap();
        let born = tx.commit(execution, &runtime).unwrap();
        let source = born.created_entity(&target("old-input")).unwrap();
        let source_commit = born.commit.commit_id;
        release_test_commit_snapshot(&runtime, &born);
        (
            Self {
                runtime,
                owner,
                installed,
                source,
                source_commit,
            },
            other,
        )
    }
    pub fn basis(&self) -> AdmittedRelationalBranchBasis {
        self.runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1
    }
    pub fn birth(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> RelationalRequiredOutputPublication {
        self.birth_for(execution, self.source, "output")
    }
    pub fn add_unrelated_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> RelationalRequiredOutputPublication {
        let mut tx = begin_main_transaction(&self.runtime);
        tx.push_batch(create("other-source"), execution.allocation_policy())
            .unwrap();
        let result = tx.commit(execution, &self.runtime).unwrap();
        let source = result.created_entity(&target("other-source")).unwrap();
        release_test_commit_snapshot(&self.runtime, &result);
        self.birth_for(execution, source, "other-output")
    }
    fn birth_for(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        source: EntityId,
        key: &str,
    ) -> RelationalRequiredOutputPublication {
        self.publish(
            execution,
            create(key),
            RequiredOutputPublication::Birth {
                admission: self.owner.admit_required_output("source-owned", [1; 32]),
                absence: None,
                meaning: RequiredOutputPublicationMeaning {
                    implementation: RequiredOutputProducingImplementation::from_checkpoint(
                        [0x11; 16], [0x22; 32],
                    )
                    .unwrap(),
                    family: "source-owned".into(),
                    source_scope: source,
                    source_partition: worth_foundational::facade::CanonicalDigestId::new([0; 32]),
                    source_epoch: worth_foundational::facade::CanonicalDigestId::new([1; 32]),
                },
                roles: vec![RequiredOutputRoleTarget {
                    role: "output".into(),
                    entity_name: "node".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Created(target(key)),
                }],
            },
        )
        .expect("single producer publication has its primary custody")
    }
    pub fn preserve(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prior: RelationalRequiredOutputPublication,
    ) -> RelationalRequiredOutputPublication {
        let admission = self
            .owner
            .admit_required_output_succession(
                self.owner.admit_required_output("source-owned", [2; 32]),
                prior.authority(),
            )
            .unwrap();
        let roles = roles(&prior, RequiredOutputRolePosture::Preserve);
        let meaning = prior.meaning().clone();
        self.publish(
            execution,
            WorkerIntentBatch::new("no-data-succession"),
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: prior,
                meaning,
                roles,
            },
        )
        .expect("single producer publication has its primary custody")
    }
    pub fn change_source(&self, execution: &ExecutionRequest<'_, '_>, value: &str) {
        let mut tx = begin_main_transaction(&self.runtime);
        tx.push_batch(
            WorkerIntentBatch::new("authored-input-edit").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: self.source,
                    fields: fields(value),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
        let committed = tx.commit(execution, &self.runtime).unwrap();
        release_test_commit_snapshot(&self.runtime, &committed);
    }
    pub fn publish(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
        publication: RequiredOutputPublication,
    ) -> Option<RelationalRequiredOutputPublication> {
        let mut tx = begin_main_transaction(&self.runtime);
        tx.push_batch(batch, execution.allocation_policy()).unwrap();
        let proposal = tx
            .validate(execution, &self.runtime)
            .unwrap()
            .bind_required_output_publication(&self.installed, publication)
            .unwrap();
        let candidate = self
            .runtime
            .prepare_validated_proposal(execution, proposal)
            .unwrap();
        let RelationalPublicationOutcome::Performed(performed) = self
            .runtime
            .publication_port()
            .compare_and_publish(execution, candidate)
        else {
            panic!("real installed publication must perform")
        };
        let result = self
            .runtime
            .settle_performed_publication(performed)
            .unwrap();
        let publication = result.required_output_publication(&self.installed);
        release_test_commit_snapshot(&self.runtime, &result);
        publication
    }
    pub fn retire(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prior: &RelationalRequiredOutputPublication,
    ) {
        let mut roles = roles(prior, RequiredOutputRolePosture::Retire);
        roles.push(RequiredOutputRoleTarget {
            role: "source".into(),
            entity_name: "node".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Retire,
            target: EntityReference::Existing(self.source),
        });
        let selected = self
            .runtime
            .readmit_authored_source_retirement(
                execution,
                &self.installed,
                &self.basis(),
                self.source,
                &roles,
            )
            .unwrap()
            .unwrap();
        let admission = self
            .owner
            .admit_authored_output_retirements(selected.authorities())
            .unwrap();
        let batch = std::iter::once(self.source)
            .chain(prior.members().map(|(_, _, _, id)| id))
            .fold(
                WorkerIntentBatch::new("authored-source-retirement"),
                |batch, entity_id| {
                    batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                        DeleteEntityIntent { entity_id },
                    )))
                },
            );
        self.publish(
            execution,
            batch,
            RequiredOutputPublication::AuthoredCascadeRetirement {
                admission,
                predecessor: selected,
            },
        );
    }
    pub fn active_holds(&self) -> u64 {
        let c = self
            .runtime
            .branch_retention_cost_counters(&self.runtime.main_branch_identity())
            .unwrap();
        c.observation_acquires - c.observation_releases
    }
    pub fn read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> worth_relational::facade::runtime::EntityReadRecord {
        let snapshot = self
            .runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let record = self
            .runtime
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .unwrap()
            .unwrap()
            .get_entity(self.source)
            .unwrap()
            .clone();
        self.runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .unwrap();
        record
    }
    pub fn reopen(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prior: &RelationalRequiredOutputPublication,
    ) -> (Self, RelationalRequiredOutputPublication) {
        let checkpoint = self
            .runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap();
        let mut runtime = runtime();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(RelationalSchemaRegistry::new())
            .unwrap();
        runtime
            .durability_recovery()
            .restore_native_checkpoint(execution, &checkpoint)
            .unwrap();
        let receipt = runtime
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let installed =
            installed.with_source_scope_owned_producers([("source-owned".into(), KindId(1))]);
        let cold = Self {
            runtime,
            owner,
            installed,
            source: self.source,
            source_commit: self.source_commit,
        };
        let restored = cold
            .runtime
            .readmit_current_owned_output_successor(
                execution,
                &cold.installed,
                &cold.basis(),
                prior.producer(),
                prior.meaning(),
            )
            .unwrap()
            .expect("the checkpoint preserves this exact current route");
        (cold, restored)
    }
}
