//! Forked aggregate worlds: commits and bounded observations on one exact
//! product branch.

use std::cell::Cell;

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateContribution;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateWorld;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::SourceAmount;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::SourceIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantAggregateDenialKind;
use worth_query_execution::facade::product::WorthQueryProductBranch;

impl AggregateWorld {
    pub fn main_branch(&self) -> WorthQueryProductBranch {
        self._runtime.current_world()
    }

    pub fn fork(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> WorthQueryProductBranch {
        self._runtime
            .branches()
            .fork(branch)
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(phase_custody)
            .expect("the fork publishes")
    }

    /// The aggregate as a bounded projection at the branch's relational head
    /// observes it. Raw test commits move that head, not the product binding.
    pub fn observe_on(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(i64, u64), WorthQueryInvariantAggregateDenialKind> {
        let observed = Cell::new(None);
        let identity = self.relational_identity(phase_custody, branch);
        let (_, basis) = (*self.authority.graph_for_test()).with_runtime_mut_for_test(
            &phase_custody.borrowed_request_for_test(),
            |runtime| {
                runtime
                    .observe_branch(&identity)
                    .expect("the branch head observes")
            },
        );
        self.authority
            .project_with_work_limit_for_test(
                &phase_custody.borrowed_request_for_test(),
                Some(10_000),
                basis,
                |reader| {
                    let result = reader.summarize_exclusive_incoming(
                        AggregateContribution::reference(),
                        SourceAmount::at("SourceAmount"),
                        &self.target,
                    );
                    observed.set(Some(result.as_ref().map_or_else(
                        |denial| Err(denial.kind()),
                        |aggregate| Ok((*aggregate.value(), aggregate.source_count())),
                    )));
                    result
                },
                None,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the branch projection completes");
        observed.get().expect("the aggregate was observed")
    }

    pub fn source(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        source: &str,
    ) -> EntityId {
        (*self
            .authority
            .project(
                phase_custody,
                |reader| {
                    reader.resolve_entity(SourceIdentity::at("SourceIdentity"), source.to_owned())
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("source projection")
            .output()
            .as_ref()
            .expect("source identity resolves")
            .entity_id_for_test())
    }

    /// The one contribution a source makes to the target.
    pub fn contribution(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        source: EntityId,
    ) -> RelationId {
        (*self
            .authority
            .project(
                phase_custody,
                |reader| reader.relations_to(AggregateContribution::reference(), &self.target),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("contribution projection")
            .output()
            .as_ref()
            .expect("contributions read")
            .iter()
            .find(|relation| (*(*relation.from_for_test()).entity_id_for_test()) == source)
            .expect("the source contributes")
            .relation_id_for_test())
    }

    pub fn remove_relation_on(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        relation_id: RelationId,
    ) {
        self.commit_on(
            phase_custody,
            branch,
            "aggregate-remove-contribution",
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id,
            })),
        );
    }

    pub fn commit_on(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        label: &'static str,
        intent: MutationIntent,
    ) {
        let identity = self.relational_identity(phase_custody, branch);
        let execution = phase_custody.borrowed_request_for_test();
        (*self.authority
            .graph_for_test())
            .with_runtime_mut_for_test(&execution, |runtime| {
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
                    .push_batch(
                        WorkerIntentBatch::new(label).push(intent),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("test staging stays within configured resource budgets");
                let committed = transaction
                    .commit(&execution, runtime)
                    .expect("the branch commits");
                worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
                    runtime,
                    &committed.snapshot,
                );
            });
    }

    pub fn relational_identity(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> worth_relational::facade::branch::RelationalBranchIdentity {
        let selected = self
            ._runtime
            .on_branch(branch)
            .select(phase_custody)
            .expect("the branch remains selectable");
        let branch_id = selected
            .product()
            .observation_for_test()
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id()
            .clone();
        (*self.authority.graph_for_test()).with_runtime(
            &phase_custody.borrowed_request_for_test(),
            |runtime| {
                runtime
                    .branch_identity(&branch_id)
                    .expect("the branch's relational identity is installed")
            },
        )
    }
}
