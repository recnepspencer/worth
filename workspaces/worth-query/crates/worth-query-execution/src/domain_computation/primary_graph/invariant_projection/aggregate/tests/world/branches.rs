//! Forked aggregate worlds: commits and bounded observations on one exact
//! product branch.

use std::cell::Cell;

use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::transactions::{
    DeleteRelationIntent, MutationIntent, RelationMutationIntent, WorkerIntentBatch,
};

use super::super::super::WorthQueryInvariantAggregateDenialKind;
use super::{AggregateContribution, AggregateWorld, SourceAmount, SourceIdentity};
use crate::basis::WorthQueryProductBranch;

impl AggregateWorld {
    pub(in super::super) fn main_branch(&self) -> WorthQueryProductBranch {
        self._runtime.current_world()
    }

    pub(in super::super) fn fork(
        &self,
        branch: WorthQueryProductBranch,
    ) -> WorthQueryProductBranch {
        self._runtime
            .branches()
            .fork(branch)
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create()
            .expect("the fork publishes")
    }

    /// The aggregate as a bounded projection at the branch's relational head
    /// observes it. Raw test commits move that head, not the product binding.
    pub(in super::super) fn observe_on(
        &self,
        branch: WorthQueryProductBranch,
    ) -> Result<(i64, u64), WorthQueryInvariantAggregateDenialKind> {
        let observed = Cell::new(None);
        let identity = self.relational_identity(branch);
        let (_, basis) = self.authority.graph.with_runtime_mut(|runtime| {
            runtime
                .observe_branch(&identity)
                .expect("the branch head observes")
        });
        self.authority
            .project_bounded(10_000, basis, |reader| {
                let result = reader.summarize_exclusive_incoming(
                    AggregateContribution::reference(),
                    SourceAmount::reference(),
                    &self.target,
                );
                observed.set(Some(result.as_ref().map_or_else(
                    |denial| Err(denial.kind()),
                    |aggregate| Ok((*aggregate.value(), aggregate.source_count())),
                )));
                result
            })
            .expect("the branch projection completes");
        observed.get().expect("the aggregate was observed")
    }

    pub(in super::super) fn source(&self, source: &str) -> EntityId {
        self.authority
            .project(|reader| reader.resolve_entity(SourceIdentity::reference(), source.to_owned()))
            .expect("source projection")
            .output()
            .as_ref()
            .expect("source identity resolves")
            .entity_id
    }

    /// The one contribution a source makes to the target.
    pub(in super::super) fn contribution(&self, source: EntityId) -> RelationId {
        self.authority
            .project(|reader| reader.relations_to(AggregateContribution::reference(), &self.target))
            .expect("contribution projection")
            .output()
            .as_ref()
            .expect("contributions read")
            .iter()
            .find(|relation| relation.from.entity_id == source)
            .expect("the source contributes")
            .relation_id
    }

    pub(in super::super) fn remove_relation_on(
        &self,
        branch: WorthQueryProductBranch,
        relation_id: RelationId,
    ) {
        self.commit_on(
            branch,
            "aggregate-remove-contribution",
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id,
            })),
        );
    }

    pub(in super::super) fn commit_on(
        &self,
        branch: WorthQueryProductBranch,
        label: &'static str,
        intent: MutationIntent,
    ) {
        let identity = self.relational_identity(branch);
        self.authority.graph.with_runtime_mut(|runtime| {
            let admitted = runtime
                .admit_branch_basis(&identity)
                .expect("the branch binding admits");
            let mut transaction = runtime
                .begin_branch_transaction(
                    &admitted,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("owner-admitted transaction context");
            transaction
                .push_batch(WorkerIntentBatch::new(label).push(intent))
                .expect("test staging stays within configured resource budgets");
            let committed = transaction.commit(runtime).expect("the branch commits");
            crate::relational_snapshot_release::release_query_snapshot(
                runtime,
                &committed.snapshot,
            );
        });
    }

    fn relational_identity(
        &self,
        branch: WorthQueryProductBranch,
    ) -> worth_relational::facade::branch::RelationalBranchIdentity {
        let selected = self
            ._runtime
            .on_branch(branch)
            .select()
            .expect("the branch remains selectable");
        let branch_id = selected
            .product()
            .observation()
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id()
            .clone();
        self.authority.graph.with_runtime(|runtime| {
            runtime
                .branch_identity(&branch_id)
                .expect("the branch's relational identity is installed")
        })
    }
}
