//! Closed passive observations of the actual installed owners for the Host court.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    runtime::{EntityReadRecord, RelationReadRecord},
    transactions::RelationalExecutionDenialCause,
};

/// Descriptive history only: no admission, retained basis, or history hold.
pub struct WorthQueryOracleOccurrenceForTest {
    pub identity: worth_runtime_world::facade::CompositeCommitIdentity,
    pub parent: worth_runtime_world::facade::CompositeCommitParent,
    pub provenance: worth_runtime_world::facade::CompositeCommitProvenance,
    pub relational_publication: Option<worth_relational::facade::history::RelationalCommitIdentity>,
    pub relational_fork: Option<worth_relational::facade::history::BranchId>,
    pub relational_change: worth_runtime_world::facade::CompositeComponentChangePosture,
    pub signal_change: worth_runtime_world::facade::CompositeComponentChangePosture,
    // The Signal admission token is process-local and deliberately excluded.
    pub signal_movement: Option<WorthQueryOracleSignalMovementForTest>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOracleSignalMovementForTest {
    Advanced,
    Forked,
}
pub struct WorthQueryOracleHistoryPageForTest {
    pub occurrences: Vec<WorthQueryOracleOccurrenceForTest>,
    pub complete: bool,
    pub crossed_retired: bool,
}

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Copies descriptive graph truth, never the Native receiver or a writer.
    #[doc(hidden)]
    pub fn application_oracle_graph_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Vec<EntityReadRecord>,
            Vec<RelationReadRecord>,
            worth_relational::facade::symbols::SymbolTableSnapshot,
        ),
        RelationalExecutionDenialCause,
    > {
        self.primary_provider
            .graph
            .with_runtime(execution, |runtime| {
                let branch = runtime
                    .branch_identity(self.relational_branch_identity.branch_id())
                    .expect("the inspected application branch remains registered");
                let (_, basis) = runtime
                    .observe_branch(&branch)
                    .expect("the inspected application branch remains observable");
                let snapshot = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .expect("the actual inspected branch admits its read snapshot");
                let read = runtime.read_truth().read_snapshot(&snapshot, execution);
                let answer = read.map(|view| {
                    let view = view.expect("the admitted inspected snapshot remains readable");
                    let symbols = {
                        let view = runtime
                            .read_truth()
                            .project_observation(&basis.observation())
                            .expect("actual inspected root");
                        let pin = runtime
                            .read_truth()
                            .pin_view_read_capability(&view, &[], execution)
                            .expect("actual inspected symbols");
                        pin.symbols().snapshot()
                    };

                    (view.entities().to_vec(), view.relations().to_vec(), symbols)
                });
                runtime
                    .snapshots()
                    .release_snapshot(&snapshot)
                    .expect("the same receiver releases its actual inspection snapshot");
                answer
            })
    }
    /// Detached recorded data; no component admission or history hold escapes.
    #[doc(hidden)]
    pub fn application_oracle_head_for_test(
        &self,
    ) -> worth_foundational::facade::RecordedProductBranchReference {
        let head = self
            .product_runtime
            .owner
            .observation_port()
            .observe_root_product_branch()
            .expect("the inspected World remains available");
        describe_head(&head)
    }
    /// Ordered identity/parent/provenance data, not the retention-bearing traversal.
    #[doc(hidden)]
    pub fn application_oracle_ancestry_for_test(&self) -> WorthQueryOracleHistoryPageForTest {
        let head = self
            .product_runtime
            .owner
            .observation_port()
            .observe_root_product_branch()
            .expect("the inspected World remains available");
        let history = self
            .product_runtime
            .owner
            .inspection_port()
            .trace_ancestry(
                head.selected_commit().clone(),
                std::num::NonZeroUsize::new(256).unwrap(),
            )
            .expect("the bounded oracle history remains inspectable");
        WorthQueryOracleHistoryPageForTest {
            complete: history.is_complete(),
            crossed_retired: history.crossed_retired_history(),
            occurrences: history
                .commits()
                .map(|c| WorthQueryOracleOccurrenceForTest {
                    identity: c.identity().clone(),
                    parent: c.parent().clone(),
                    provenance: c.provenance().clone(),
                    relational_publication: c.relational_publication_identity().cloned(),
                    relational_fork: c
                        .relational_fork_target_identity()
                        .map(|b| b.branch_id().clone()),
                    relational_change: c.relational_change(),
                    signal_change: c.signal_change(),
                    signal_movement: c.signal_publication_identity().map(|identity| {
                        use worth_runtime_world::facade::CompositeSignalPublicationIdentity as S;
                        match identity {
                            S::Advanced(_) => WorthQueryOracleSignalMovementForTest::Advanced,
                            S::Forked(_) => WorthQueryOracleSignalMovementForTest::Forked,
                        }
                    }),
                })
                .collect(),
        }
    }
}
fn describe_head(
    head: &worth_runtime_world::facade::ProductBranchObservation,
) -> worth_foundational::facade::RecordedProductBranchReference {
    use worth_foundational::facade::*;
    let name = head.branch_identity().name().as_str().to_owned();
    match head.owner_identity().durable() {
        Some(owner) => RecordedProductBranchReference::Home(
            DurableProductBranchReference::from_recorded(
                DurableProductBranchIdentity::from_recorded(owner, name)
                    .expect("owner branch name"),
                DurableProductBranchIncarnation::from_recorded(
                    owner,
                    head.lifecycle_incarnation().ordinal(),
                ),
                DurableProductReferenceGeneration::from_recorded(head.reference_generation().get()),
                DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    head.selected_commit().ordinal(),
                ),
            )
            .expect("the actual owner scopes all of its observed head"),
        ),
        None => RecordedProductBranchReference::Memory(
            MemoryProductBranchReference::from_observed(
                head.owner_identity().get(),
                name,
                head.lifecycle_incarnation().ordinal(),
                head.reference_generation().get(),
                head.selected_commit().ordinal(),
            )
            .expect("owner branch name"),
        ),
    }
}
