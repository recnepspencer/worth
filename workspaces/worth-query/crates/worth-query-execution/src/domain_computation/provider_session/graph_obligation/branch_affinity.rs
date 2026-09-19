use crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_bridge::facade::TruthBranchIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryGraphWorkBranchAffinity {
    relational: BranchId,
    truth: TruthBranchIdentity,
}

impl WorthQueryGraphWorkBranchAffinity {
    pub(super) fn from_query_basis(basis: &WorthQueryApplicationBasisIdentity) -> Self {
        Self::from_relational_branch(basis.branch_id().clone())
    }

    pub(super) fn from_snapshot(snapshot: &SnapshotHandle) -> Self {
        Self::from_relational_branch(snapshot.branch_id().clone())
    }

    pub(super) fn from_product(product: &crate::basis::WorthQueryProductObservationLease) -> Self {
        Self::from_relational_branch(product.relational_basis_descriptor().branch_id().clone())
    }

    fn from_relational_branch(relational: BranchId) -> Self {
        let truth = TruthBranchIdentity::from_relational_branch_id(relational.0.clone());
        Self { relational, truth }
    }

    pub(in crate::domain_computation) const fn relational(&self) -> &BranchId {
        &self.relational
    }

    pub(in crate::domain_computation) const fn truth(&self) -> &TruthBranchIdentity {
        &self.truth
    }

    pub(super) fn admits_query_basis(&self, basis: &WorthQueryApplicationBasisIdentity) -> bool {
        basis.branch_id() == &self.relational
    }

    pub(super) fn admits_snapshot(&self, snapshot: &SnapshotHandle) -> bool {
        snapshot.branch_id() == &self.relational
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use worth_relational::facade::{
        history::BranchId,
        identity::{KindId, PartitionId},
        runtime::RelationalRuntimeApi,
        schema::{
            EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry,
            SchemaId, SchemaVersionId,
        },
        symbols::ClientKey,
        transactions::{
            AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
        },
    };
    use worth_runtime_bridge::facade::TruthBranchIdentity;

    use super::WorthQueryGraphWorkBranchAffinity;

    #[test]
    fn equal_version_snapshot_from_another_branch_cannot_satisfy_affinity() {
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(
                RelationalSchemaRegistry::new()
                    .register_entity_kind(EntityKindRegistration {
                        kind_id: KindId(1),
                        kind_name: "branch-affinity.fixture".into(),
                        schema_id: SchemaId("branch-affinity-fixture".into()),
                        schema_version_id: SchemaVersionId(1),
                        aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
                    })
                    .expect("branch-affinity fixture schema registers"),
            )
            .build();
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
            .push_batch(WorkerIntentBatch::new("branch-affinity-fixture").push(
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("branch-affinity-root"),
                    fields: AspectFieldPatch::new(BTreeMap::new()),
                })),
            ))
            .expect("test staging stays within configured resource budgets");
        let committed = transaction
            .commit(&runtime)
            .expect("branch-affinity fixture root commits");
        assert!(runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .is_ok());
        let admitted_branch = BranchId("main".to_owned());
        let substitute_branch = BranchId("hostile".to_owned());
        let (_, fork_basis) = runtime
            .observe_fork_source(&admitted_branch)
            .expect("main branch exposes an exact fork basis");
        runtime
            .fork_branch(substitute_branch.clone(), fork_basis)
            .expect("main basis can fork a sibling branch");
        let admitted_identity = runtime.branch_identity(&admitted_branch).unwrap();
        let substitute_identity = runtime.branch_identity(&substitute_branch).unwrap();
        let (_, admitted_basis) = runtime.observe_branch(&admitted_identity).unwrap();
        let (_, substitute_basis) = runtime.observe_branch(&substitute_identity).unwrap();
        let admitted = runtime
            .snapshots()
            .snapshot_for_observation(&admitted_basis.observation())
            .unwrap();
        let substitute = runtime
            .snapshots()
            .snapshot_for_observation(&substitute_basis.observation())
            .unwrap();
        let affinity = WorthQueryGraphWorkBranchAffinity::from_snapshot(&admitted);

        assert!(affinity.admits_snapshot(&admitted));
        assert!(!affinity.admits_snapshot(&substitute));
        assert_eq!(affinity.relational(), admitted.branch_id());
        assert_eq!(
            affinity.truth(),
            &TruthBranchIdentity::from_relational_branch_id(admitted.branch_id().0.clone())
        );
        assert!(runtime.snapshots().release_snapshot(&admitted).is_ok());
        assert!(runtime.snapshots().release_snapshot(&substitute).is_ok());
    }
}
