use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::{
    InvariantCatalog, InvariantRegistration, InvariantRule, RuntimeBridge,
    WorthQueryBackendAdmissibleMutation, WorthQueryRuntimeWriteAuthorityAdapter,
    WriteAuthorityExecutionReceipt,
};

use super::super::support::TestWriteAuthority;

const HOST_SENTINEL_KIND: KindId = KindId(1);
const HOST_RUNTIME_NAME: &str = "primary-graph-composition-host";

pub(super) fn host_relational_runtime() -> RelationalRuntime {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(host_entity_registration(
            HOST_SENTINEL_KIND,
            "host.sentinel",
            "host",
        ))
        .expect("host sentinel schema should register");
    RelationalRuntimeApi::builder()
        .runtime_name(HOST_RUNTIME_NAME)
        .schema_registry(schema)
        .invariant_catalog(host_invariant_catalog())
        .build()
}

pub(super) fn mixed_basis_relational_runtime() -> RelationalRuntime {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(host_entity_registration(KindId(1), "host.left", "host-a"))
        .and_then(|registry| {
            registry.register_entity_kind(host_entity_registration(
                KindId(2),
                "host.right",
                "host-b",
            ))
        })
        .expect("the registry structurally admits the hostile mixed-basis fixture");
    RelationalRuntimeApi::builder()
        .runtime_name("mixed-basis-host")
        .schema_registry(schema)
        .build()
}

fn host_entity_registration(
    kind_id: KindId,
    kind_name: &str,
    schema_id: &str,
) -> EntityKindRegistration {
    EntityKindRegistration {
        kind_id,
        kind_name: kind_name.to_string(),
        schema_id: SchemaId(schema_id.to_string()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: KindAspectContractDeclarations::default(),
    }
}

fn host_invariant_catalog() -> InvariantCatalog {
    InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::MaxMergedIntents(17),
        )],
    }
}

pub(super) struct CommittingWriteAuthority;

impl WorthQueryRuntimeWriteAuthorityAdapter for CommittingWriteAuthority {
    fn write(
        &mut self,
        bridge: &RuntimeBridge,
        relational_runtime: Option<&mut RelationalRuntime>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError> {
        let runtime = relational_runtime.ok_or_else(|| {
            WorthQueryWorkspaceError::new(
                "primary graph composition write did not receive the shared Relational runtime",
            )
        })?;
        if runtime.config().execution.runtime_name != HOST_RUNTIME_NAME {
            return Err(WorthQueryWorkspaceError::new(
                "primary graph publication replaced the host Relational configuration",
            ));
        }
        if runtime.config().schema.invariant_catalog != host_invariant_catalog() {
            return Err(WorthQueryWorkspaceError::new(
                "primary graph publication discarded the host Relational invariant catalog",
            ));
        }
        runtime
            .config()
            .schema
            .registry
            .resolve_entity(HOST_SENTINEL_KIND)
            .map_err(|_| {
                WorthQueryWorkspaceError::new(
                    "primary graph publication discarded the host Relational schema",
                )
            })?;

        let mut transaction = {
            let transaction_validation_input = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .expect("main branch binding");
            runtime
                .begin_branch_transaction(
                    &transaction_validation_input,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("owner-admitted transaction context")
        };
        transaction
            .push_batch(
                WorkerIntentBatch::new("ordinary-query-shared-root-proof").push(
                    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: HOST_SENTINEL_KIND,
                        client_key: ClientKey::raw("ordinary-query-write"),
                        fields: AspectFieldPatch::default(),
                    })),
                ),
                worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
        let committed = transaction
            .commit(
                runtime,
                worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(|error| {
                WorthQueryWorkspaceError::new(format!(
                    "ordinary Query write could not commit to the shared graph: {error:?}"
                ))
            })?;
        runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .expect("ordinary Query write snapshot should close exactly once");

        let mut receipt_authority = TestWriteAuthority;
        receipt_authority.write(bridge, Some(runtime), mutation)
    }
}
