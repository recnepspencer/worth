use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    identity::{EntityId, KindId, PartitionId},
    mvcc::{PreparedRelationalCommitCandidate, RelationalTransactionIntent},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

pub struct CargoRecords {
    pub runtime: Arc<RelationalRuntime>,
    pub grain: EntityId,
}

pub fn key() -> AspectKey {
    AspectKey::new("value").unwrap()
}
pub fn field() -> FieldKey {
    FieldKey::new("value").unwrap()
}
pub fn contract() -> AspectContract {
    aspects()
        .contract()
        .for_key(key())
        .identified_by(AspectIdentity(9172))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String)
}
fn patch(value: &str) -> AspectFieldPatch {
    AspectFieldPatch::from(BTreeMap::from([(
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            key(),
            CanonicalFieldPath::single(field()),
        ),
        AspectValue::String(value.to_owned().into()),
    )]))
}
impl CargoRecords {
    pub fn install() -> Self {
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "cargo".into(),
                schema_id: SchemaId("publication-example".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::EntityField { field: field() },
                        contract: contract(),
                    },
                ]),
            })
            .unwrap();
        let runtime = Arc::new(
            RelationalRuntimeApi::builder()
                .schema_registry(registry)
                .build(),
        );
        let basis = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1;
        let mut tx = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(WorkerIntentBatch::new("grain").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("grain"),
                fields: patch("4"),
            }),
        )))
        .unwrap();
        let result = tx.commit(&runtime).unwrap();
        let grain = result
            .changed_records
            .iter()
            .find_map(|record| match record {
                RecordRef::Entity(id) => Some(*id),
                _ => None,
            })
            .unwrap();
        Self { runtime, grain }
    }
    pub fn candidate(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        value: &str,
    ) -> PreparedRelationalCommitCandidate {
        let mut tx = self
            .runtime
            .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new("update-grain").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: self.grain,
                    fields: patch(value),
                }),
            )),
        )
        .unwrap();
        self.runtime
            .preparation_port()
            .prepare_branch_transaction(tx)
            .unwrap()
    }
}
