use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::{EntityId, KindId, PartitionId},
    mvcc::{PreparedRelationalCommitCandidate, RelationalTransactionIntent},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

pub struct CargoRecords {
    pub runtime: Arc<RelationalRuntime>,
    pub ids: BTreeMap<&'static str, EntityId>,
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
    pub fn install(include_steel: bool) -> Self {
        let mut registry = RelationalSchemaRegistry::new();
        for (kind, name) in [(1, "port"), (2, "voyage"), (3, "manifest"), (4, "cargo")] {
            registry = registry
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(kind),
                    kind_name: name.into(),
                    schema_id: SchemaId("supply-chain".into()),
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                        DeclaredAspectContractBinding {
                            binding: AspectBinding::EntityField { field: field() },
                            contract: contract(),
                        },
                    ]),
                })
                .expect("court declaration: entity schema");
        }
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(5),
                kind_name: "carries".into(),
                schema_id: SchemaId("supply-chain".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
            .expect("court declaration: relation schema");
        let runtime = Arc::new(
            RelationalRuntimeApi::builder()
                .schema_registry(registry)
                .build(),
        );
        let mut ids = BTreeMap::new();
        for (name, kind, value) in [
            ("harbor", 1, "open"),
            ("voyage", 2, "12"),
            ("manifest", 3, "harbor"),
            ("grain", 4, "4"),
            ("steel", 4, "6"),
        ] {
            let basis = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .1;
            let mut tx = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            tx.push_batch(WorkerIntentBatch::new(name).push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(kind),
                    client_key: ClientKey::raw(name),
                    fields: patch(value),
                }),
            )))
            .expect("component owner: stage entity");
            let result = tx
                .commit(&runtime)
                .expect("component owner: install entity");
            let id = result
                .changed_records
                .iter()
                .find_map(|r| match r {
                    RecordRef::Entity(id) => Some(*id),
                    _ => None,
                })
                .unwrap();
            ids.insert(name, id);
        }
        for (source, target) in [
            ("voyage", "manifest"),
            ("manifest", "grain"),
            ("manifest", "steel"),
            ("manifest", "harbor"),
        ] {
            if target == "steel" && !include_steel {
                continue;
            }
            let basis = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .1;
            let mut tx = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            tx.push_batch(WorkerIntentBatch::new("link").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(5),
                    client_key: ClientKey::raw(format!("{source}-{target}")),
                    source: EntityReference::Existing(ids[source]),
                    target: EntityReference::Existing(ids[target]),
                    fields: AspectFieldPatch::default(),
                }),
            )))
            .expect("component owner: stage route link");
            tx.commit(&runtime)
                .expect("component owner: install route link");
        }
        Self { runtime, ids }
    }
    pub fn read(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> super::observation::SupplyChainObservation {
        let pinned = self
            .runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .expect("observation: exact snapshot");
        let values = {
            let read = self
                .runtime
                .read_truth()
                .read_snapshot(&pinned)
                .expect("observation: relational read");
            let names: BTreeMap<_, _> = self
                .ids
                .iter()
                .map(|(name, id)| (*id, (*name).to_owned()))
                .collect();
            let links = read
                .relations()
                .iter()
                .map(|relation| {
                    (
                        names[&relation.source].clone(),
                        names[&relation.target].clone(),
                    )
                })
                .collect();
            let records = self
                .ids
                .iter()
                .map(|(name, id)| {
                    let entity = read.get_entity(*id).expect("observation: installed entity");
                    let value = entity
                        .authoritative_aspect_state
                        .as_ref()
                        .unwrap()
                        .get(&key())
                        .unwrap()
                        .view();
                    let ContractValidatedAspectValueView::Scalar(AspectValue::String(
                        InternedString::Raw(value),
                    )) = value
                    else {
                        panic!("observation: scalar string")
                    };
                    ((*name).to_owned(), value.to_string())
                })
                .collect();
            super::observation::SupplyChainObservation { records, links }
        };
        self.runtime
            .snapshots()
            .release_snapshot(&pinned)
            .expect("observation: release exact read");
        values
    }
    pub fn candidate(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        name: &str,
        value: &str,
    ) -> PreparedRelationalCommitCandidate {
        self.candidate_with_control(
            basis,
            name,
            value,
            worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
        )
    }
    pub fn candidate_with_control(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        name: &str,
        value: &str,
        control: worth_relational::facade::mvcc::RelationalOperationControl,
    ) -> PreparedRelationalCommitCandidate {
        let mut tx = self
            .runtime
            .begin_branch_transaction_with_control(
                basis,
                RelationalTransactionIntent::ordinary(),
                control,
            )
            .expect("component owner: begin exact update");
        tx.push_batch(
            WorkerIntentBatch::new("update-cargo").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: self.ids[name],
                    fields: patch(value),
                }),
            )),
        )
        .expect("component owner: stage update");
        self.runtime
            .preparation_port()
            .prepare_branch_transaction(tx)
            .expect("component owner: prepare update")
    }
}
