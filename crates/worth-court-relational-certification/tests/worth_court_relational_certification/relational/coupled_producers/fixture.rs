//! Real Native installation, authored carriers and atomic producer participants.
use super::*;
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::schema::*;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;

mod content;
mod derived_law;
mod optional_struct;
mod replay;
pub use content::{attachments, content_create};
pub use derived_law::USES;
use derived_law::{Extra, BARE};
use optional_struct::*;

const RETAIN: CascadeDeletePolicy = CascadeDeletePolicy::RetainDanglingForAudit;
const CASCADE: CascadeDeletePolicy = CascadeDeletePolicy::CascadeDeleteRelations;

pub struct World {
    pub runtime: RelationalRuntime,
    pub owner: worth_proof::InstalledApplicationPublicationOwner,
    pub installed: RelationalInstalledApplicationPublicationAuthority,
    pub source: EntityId,
    pub carrier: EntityId,
    /// Declares the Struct aspect `summary`; births then write its `title`.
    pub summary: bool,
    /// Declares only the Optional Struct aspect `summary` (no `LifecycleTransition`).
    pub optional_struct_only: bool,
    /// The schema this world installed, when known, for a cold WAL replay.
    pub registry: Option<RelationalSchemaRegistry>,
}
impl World {
    pub fn new(execution: &ExecutionRequest<'_, '_>, storage: bool, owns_source: bool) -> Self {
        Self::with_aspects(
            execution,
            storage,
            owns_source,
            false,
            false,
            false,
            BARE,
            RETAIN,
        )
    }
    pub fn new_with_status(execution: &ExecutionRequest<'_, '_>) -> Self {
        Self::with_aspects(execution, true, true, true, false, false, BARE, RETAIN)
    }
    /// An owned-source world whose relation kind is deleted with an endpoint.
    pub fn new_cascading(execution: &ExecutionRequest<'_, '_>) -> Self {
        Self::with_aspects(execution, true, true, false, false, false, BARE, CASCADE)
    }
    /// An owned-source world whose kind also declares the Struct aspect `summary`.
    pub fn new_with_summary(execution: &ExecutionRequest<'_, '_>) -> Self {
        let mut world = Self::with_aspects(execution, true, true, false, true, false, BARE, RETAIN);
        world.summary = true;
        world
    }
    /// An owned-source world without `LifecycleTransition`, declaring only the
    /// Optional Struct aspect `summary`, matching Query-compiled fieldless kinds.
    pub fn new_optional_struct_only(execution: &ExecutionRequest<'_, '_>) -> Self {
        let mut world = Self::with_aspects(execution, true, true, false, true, true, BARE, RETAIN);
        world.summary = true;
        world.optional_struct_only = true;
        world
    }
    pub fn with_invariant(
        execution: &ExecutionRequest<'_, '_>,
        rule: CustomInvariantRegistration,
    ) -> Self {
        Self::with_aspects(
            execution,
            true,
            true,
            false,
            false,
            false,
            Extra::rule(rule),
            RETAIN,
        )
    }
    fn with_aspects(
        execution: &ExecutionRequest<'_, '_>,
        storage: bool,
        owns_source: bool,
        with_status: bool,
        with_summary: bool,
        optional_struct_only: bool,
        extra: Extra,
        cascade: CascadeDeletePolicy,
    ) -> Self {
        let mut fixture = if optional_struct_only {
            AspectSchemaFixture::default()
        } else {
            AspectSchemaFixture::with_default_declared_aspects(cascade)
        };
        if with_status {
            let (binding, contract) =
                worth_testkit_relational::aspect_storage::definition::entity_field_aspect(
                    aspect_key("status"),
                    field_key("status"),
                );
            fixture
                .entity_aspects
                .push(DeclaredAspectContractBinding { binding, contract });
        }
        if with_summary {
            let (binding, contract) = if optional_struct_only {
                optional_summary_struct_aspect()
            } else {
                worth_testkit_relational::aspect_storage::definition::entity_summary_struct_aspect(
                    aspect_key("summary"),
                    field_key("summary"),
                )
            };
            fixture
                .entity_aspects
                .push(DeclaredAspectContractBinding { binding, contract });
        }
        let registry = extra.extend(fixture.build_registry(), &fixture);
        let registry = if storage {
            register_current_publication_storage(
                registry,
                &fixture.schema_id,
                fixture.schema_version_id,
            )
            .unwrap()
        } else {
            registry
        };
        let builder = RelationalRuntimeApi::builder().schema_registry(registry.clone());
        let builder = if let Some(rule) = extra.rule {
            builder.custom_invariant(rule)
        } else {
            builder
        };
        let mut runtime = builder.build().unwrap();
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
            if optional_struct_only {
                WorkerIntentBatch::new("optional-struct-sources")
                    .push(optional_struct_create_intent("source", "source"))
                    .push(optional_struct_create_intent("carrier", "carrier"))
            } else if with_status {
                WorkerIntentBatch::new("two-field-sources")
                    .push(create_with_status("source"))
                    .push(create_with_status("carrier"))
            } else {
                create("source").push(create_intent("carrier"))
            },
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
            summary: false,
            optional_struct_only: false,
            registry: Some(registry),
        }
    }
    pub fn birth(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        key: &str,
    ) -> RelationalRequiredOutputPublication {
        let mut tx = begin_main_transaction(&self.runtime);
        let batch = if self.summary {
            WorkerIntentBatch::new(key).push(summary_create_intent(key, "title-0"))
        } else {
            create(key)
        };
        tx.push_batch(batch, execution.allocation_policy()).unwrap();
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
            source_partition: worth_foundational::CanonicalDigestId::new([0; 32]),
            source_epoch: worth_foundational::CanonicalDigestId::new([1; 32]),
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
        self.prepare_with_owners(execution, owners, None, assembly)
    }
    /// Prepare an edit envelope: the authored prefix edits `source`, and
    /// Native admits `later` as the sources of the authored stages after it.
    pub fn prepare_stages(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        later: &[EntityId],
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let owners = self.runtime.readmit_edit_coupled_stage_owners(
            execution,
            &self.installed,
            &self.basis(),
            self.source,
            later,
        )?;
        self.prepare_with_owners(execution, owners, None, assembly)
    }
    fn prepare_with_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        owners: RelationalAuthoredCoupledOwners,
        prefix: Option<WorkerIntentBatch>,
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let mut tx = begin_main_transaction(&self.runtime);
        let edit = if self.optional_struct_only {
            WorkerIntentBatch::new("summary-source-edit").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: self.source,
                    fields: string_aspect_field_patch([(
                        aspect_key("summary"),
                        field_key("title"),
                        "edited-source",
                    )]),
                }),
            ))
        } else {
            update(self.source, "edited-source")
        };
        tx.push_batch(edit, execution.allocation_policy()).unwrap();
        if let Some(prefix) = prefix {
            tx.push_batch(prefix, execution.allocation_policy())
                .unwrap();
        }
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
/// A create writing `name` and the Struct field `summary.title`.
pub fn summary_create_intent(key: &str, title: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: string_aspect_field_patch([
            (aspect_key("name"), field_key("name"), key),
            (aspect_key("summary"), field_key("title"), title),
        ]),
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

fn create_with_status(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: string_aspect_field_patch([
            (aspect_key("name"), field_key("name"), key),
            (aspect_key("status"), field_key("status"), "initial"),
        ]),
    }))
}
