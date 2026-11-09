//! Real Native installation, authored carriers and atomic producer participants.
use super::*;
use worth_relational::facade::schema::*;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;

pub struct World {
    pub runtime: RelationalRuntime,
    pub owner: worth_proof::InstalledApplicationPublicationOwner,
    pub installed: RelationalInstalledApplicationPublicationAuthority,
    pub source: EntityId,
    pub carrier: EntityId,
}
impl World {
    pub fn new(execution: &ExecutionRequest<'_, '_>, storage: bool, owns_source: bool) -> Self {
        let fixture = AspectSchemaFixture::with_default_declared_aspects(
            worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
        );
        let registry = if storage {
            register_current_publication_storage(
                fixture.build_registry(),
                &fixture.schema_id,
                fixture.schema_version_id,
            )
            .unwrap()
        } else {
            fixture.build_registry()
        };
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
        let installed = if owns_source {
            installed.with_source_scope_owned_producers([("owned".into(), KindId(1))])
        } else {
            installed
        };
        let mut tx = begin_main_transaction(&runtime);
        tx.push_batch(
            create("source").push(create_intent("carrier")),
            execution.allocation_policy(),
        )
        .unwrap();
        let result = tx.commit(execution, &runtime).unwrap();
        let source = result.created_entity(&target("source")).unwrap();
        let carrier = result.created_entity(&target("carrier")).unwrap();
        release_test_commit_snapshot(&runtime, &result);
        Self {
            runtime,
            owner,
            installed,
            source,
            carrier,
        }
    }
    pub fn birth(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        key: &str,
    ) -> RelationalRequiredOutputPublication {
        let mut tx = begin_main_transaction(&self.runtime);
        tx.push_batch(create(key), execution.allocation_policy())
            .unwrap();
        let proposal = tx
            .validate(execution, &self.runtime)
            .unwrap()
            .bind_required_output_publication(
                &self.installed,
                RequiredOutputPublication::Birth {
                    admission: self.owner.admit_required_output(family, [1; 32]),
                    absence: Some(self.absence(execution, family)),
                    meaning: self.meaning(family),
                    roles: vec![created(key)],
                },
            )
            .unwrap();
        let result = self.publish(execution, proposal);
        let publication = result
            .required_output_publication(&self.installed)
            .expect("real initial publication");
        release_test_commit_snapshot(&self.runtime, &result);
        publication
    }
    pub fn basis(&self) -> AdmittedRelationalBranchBasis {
        self.runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1
    }
    pub fn meaning(&self, family: &str) -> RequiredOutputPublicationMeaning {
        RequiredOutputPublicationMeaning {
            implementation: RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(),
            family: family.into(),
            source_scope: self.source,
            source_partition: [0; 32],
            source_epoch: [1; 32],
        }
    }
    pub fn absence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
    ) -> RelationalRequiredOutputRouteAbsence {
        self.runtime
            .readmit_required_output_route_absence(
                execution,
                &self.installed,
                &self.basis(),
                &self.meaning(family),
            )
            .unwrap()
            .expect("an actually absent route")
    }
    pub fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let owners = self.runtime.readmit_edit_coupled_owners(
            execution,
            &self.installed,
            &self.basis(),
            self.source,
        )?;
        let mut tx = begin_main_transaction(&self.runtime);
        tx.push_batch(
            update(self.source, "edited-source"),
            execution.allocation_policy(),
        )
        .unwrap();
        let tx = tx.bind_authored_coupled_producers(&self.installed, owners, None)?;
        self.runtime
            .preparation_port()
            .prepare_authored_coupled_transaction(execution, tx, assembly)?
            .expect("completed real assembly")
            .validate(execution, &self.runtime)
    }
    pub fn publish(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        proposal: ValidatedRelationalProposal,
    ) -> CommitResult {
        let candidate = self
            .runtime
            .prepare_validated_proposal(execution, proposal)
            .unwrap();
        let RelationalPublicationOutcome::Performed(performed) = self
            .runtime
            .publication_port()
            .compare_and_publish(execution, candidate)
        else {
            panic!("the complete group performs")
        };
        self.runtime
            .settle_performed_publication(performed)
            .unwrap()
    }
}
pub fn target(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
    }
}
pub fn create_intent(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), key)]),
    }))
}
pub fn create(key: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(key).push(create_intent(key))
}
pub fn update(id: EntityId, value: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new("field-write").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: id,
            fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), value)]),
        }),
    ))
}
pub fn existing(id: EntityId) -> RequiredOutputRoleTarget {
    RequiredOutputRoleTarget {
        role: "carrier".into(),
        entity_name: "node".into(),
        kind: KindId(1),
        posture: RequiredOutputRolePosture::Preserve,
        target: EntityReference::Existing(id),
    }
}
pub fn created(key: &str) -> RequiredOutputRoleTarget {
    RequiredOutputRoleTarget {
        role: key.into(),
        entity_name: "node".into(),
        kind: KindId(1),
        posture: RequiredOutputRolePosture::Create,
        target: EntityReference::Created(target(key)),
    }
}
