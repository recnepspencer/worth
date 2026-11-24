//! The walk's original request and result payer over a real Branch image.
use crate::branch::root::{RelationalBranchRoot, RelationalBranchRootIdentityIssuer};
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::history::data::BranchId;
use crate::history::retention::VisibilitySnapshotBasis;
use crate::identity::data::{EntityId, KindId};
use crate::identity::data::{PartitionId, RelationId, VersionId};
use crate::runtime::state::subsystems::services::{
    instrumentation::RuntimeInstrumentation, symbol_table::RuntimeSymbolTable,
};
use crate::schema::data::{
    DescriptorSemanticsVersion, EntityKindRegistration, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::overlay::small_relation_partition::small_relation_partition_for_test;
use crate::storage::RelationalPublishedPartitionDelta;
use crate::visibility::materialization::read_records::projection::pinned::PinnedProjectionSource;
use crate::visibility::materialization::read_records::reader::query_execution::pinned_walk::{
    complete::complete, PinnedStructuralWalk, PinnedStructuralWalkDenial,
    PinnedStructuralWalkDenialKind, PinnedStructuralWalkReads, PinnedStructuralWalkScope,
    PinnedWalkEventForTest,
};
use crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::facade::RelationDirection;

fn scope(roots: &[EntityId], direction: RelationDirection) -> PinnedStructuralWalkScope<'_> {
    PinnedStructuralWalkScope::new(roots, KindId(1), KindId(2), direction)
}

#[test]
fn original_work_cancellation_and_retained_memory_survive_handoff() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let request = ExecutionRequest::serial(&host);
    let graph = WalkGraph::new(&request, true);
    let a = graph.source;
    let roots = vec![a; 4096];
    let refused = request
        .run(ExecutionWorkCeiling::new(Some(100)), |_| {
            graph.walk(&request, scope(&roots, RelationDirection::Forward))
        })
        .unwrap()
        .0
        .unwrap_err();
    assert_eq!(
        refused.kind,
        PinnedStructuralWalkDenialKind::Read(
            crate::execution::RelationalExecutionDenialCause::WorkExhausted
        ),
        "raw duplicate seeds spend original work and return no partial walk"
    );
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let observed = |event| {
        if let PinnedWalkEventForTest::Entity(id) = event {
            assert_eq!(id, a);
            cancellation.cancel();
        }
    };
    let denial = graph
        .walk(
            &ExecutionRequest::serial(&stopped),
            scope(&[a], RelationDirection::Forward).observe_for_test(&observed),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind,
        PinnedStructuralWalkDenialKind::Read(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(denial.reads.entity_records, 1);
    assert_eq!(denial.reads.adjacency_lists, 0);
    let caller_stopped = std::cell::Cell::new(false);
    let observer = |event| {
        if let PinnedWalkEventForTest::Entity(_) = event {
            caller_stopped.set(true);
        }
    };
    let control = || {
        if caller_stopped.get() {
            Err(crate::execution::RelationalExecutionDenialCause::Cancelled)
        } else {
            Ok(())
        }
    };
    let caller_denial = graph
        .walk(
            &request,
            scope(&[a], RelationDirection::Forward)
                .with_control(&control)
                .observe_for_test(&observer),
        )
        .unwrap_err();
    assert_eq!(
        caller_denial.kind,
        PinnedStructuralWalkDenialKind::Read(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(caller_denial.reads.entity_records, 1);
    assert_eq!(caller_denial.reads.adjacency_lists, 0);
    // The same original request succeeds when the separate caller control permits it.
    let walk = graph
        .walk(&request, scope(&roots, RelationDirection::Forward))
        .unwrap();
    let payer = walk.retained_memory().expect("original serial payer");
    assert!(payer.bytes() > 0);
    assert_eq!(walk.visits().len(), 2);
}

#[test]
fn optional_and_ancestral_memory_custody_uses_the_original_backing() {
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let setup = worth_harness::facade::execution_fixture::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let graph = WalkGraph::new(&setup_request, false);
    let a = graph.source;
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let unaccounted = SerialRequest::from_policy(&policy, CancellationSource::new().token(), None);
    let walk = graph
        .walk(
            &ExecutionRequest::serial(&unaccounted),
            scope(&[a], RelationDirection::Forward),
        )
        .unwrap();
    assert!(
        walk.retained_memory().is_none(),
        "absent accounting stays absent"
    );
    drop(walk);
    let parent = SerialMemoryBudget::new(Some(20_000));
    let child = SerialMemoryBudget::new(None);
    let accounted = SerialRequest::from_memory(child, CancellationSource::new().token(), None);
    let parent_host =
        SerialRequest::from_memory(parent.clone(), CancellationSource::new().token(), None);
    let walk = ExecutionRequest::serial(&parent_host)
        .run_native(|_| {
            graph.walk(
                &ExecutionRequest::serial(&accounted),
                scope(&[a], RelationDirection::Forward),
            )
        })
        .unwrap()
        .0
        .unwrap();
    let held = walk.retained_memory().unwrap().bytes();
    assert!(held > 0);
    assert_eq!(
        parent.charged_memory_bytes(),
        held,
        "scratch was released at handoff"
    );
    assert!(
        parent.reserve(20_001 - held).is_err(),
        "the held result competes on its ancestor"
    );
    drop(walk);
    assert_eq!(parent.charged_memory_bytes(), 0);
    assert_eq!(parent.reserve(20_000).unwrap().bytes(), 20_000);
}

#[test]
fn leased_walk_preserves_original_stop_partial_reads_and_output_payer() {
    use worth_execution::LeaseRequest;
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let setup = worth_harness::facade::execution_fixture::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let graph = WalkGraph::new(&setup_request, true);
    let (a, b, edge) = (graph.source, graph.target, graph.relation.unwrap());
    for stop in [true, false] {
        let cancellation = CancellationSource::new();
        let lease = worth_harness::facade::execution_fixture::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(20_000), None),
                ),
                deadline: None,
                cancellation: cancellation.token(),
            })
            .unwrap();
        let execution = ExecutionRequest::leased(&lease);
        let observer = |event| match event {
            PinnedWalkEventForTest::Relation(id) => {
                assert_eq!(id, edge);
                if stop {
                    cancellation.cancel();
                }
            }
            PinnedWalkEventForTest::Adjacency(id) => assert!([a, b].contains(&id)),
            PinnedWalkEventForTest::Entity(_) => {}
        };
        let result = graph.walk(
            &execution,
            scope(&[a], RelationDirection::Forward).observe_for_test(&observer),
        );
        if stop {
            let refusal = result.unwrap_err();
            assert_eq!(
                refusal.kind,
                PinnedStructuralWalkDenialKind::Read(
                    crate::execution::RelationalExecutionDenialCause::Cancelled
                )
            );
            assert_eq!(
                refusal.reads,
                PinnedStructuralWalkReads {
                    entity_records: 1,
                    adjacency_lists: 1,
                    relation_records: 1
                }
            );
            assert_eq!(lease.charged_memory_bytes(), 0);
        } else {
            let walk = result.unwrap();
            assert_eq!(walk.visits().len(), 2);
            assert_eq!(
                lease.charged_memory_bytes(),
                walk.retained_memory().unwrap().bytes()
            );
            drop(walk);
            assert_eq!(lease.charged_memory_bytes(), 0);
        }
    }
}

struct WalkGraph {
    basis: ResidentVisibilitySnapshotBasis,
    instrumentation: RuntimeInstrumentation,
    source: EntityId,
    target: EntityId,
    relation: Option<RelationId>,
}

impl WalkGraph {
    fn new(execution: &ExecutionRequest<'_, '_>, linked: bool) -> Self {
        let graph = small_relation_partition_for_test(PartitionId(7), linked);
        let schema = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "walk-node".into(),
                schema_id: SchemaId("walk".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: Default::default(),
            })
            .unwrap()
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "walk-edge".into(),
                schema_id: SchemaId("walk".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: Default::default(),
                relation_integrity: Default::default(),
            })
            .unwrap();
        let base =
            RelationalBranchRoot::empty_with_schema(&schema, DescriptorSemanticsVersion::default());
        let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
        journal.entity_slots.extend([0, 1]);
        if linked {
            journal.relation_slots.insert(0);
        }
        let delta = RelationalPublishedPartitionDelta::from_committed_partitions(
            &std::collections::BTreeMap::from([(
                graph.partition.partition_id,
                (graph.partition, journal),
            )]),
        );
        let symbols = RuntimeSymbolTable::default();
        let root = RelationalBranchRoot::capture_applied_candidate(
            execution,
            &RelationalBranchRootIdentityIssuer::default(),
            &base,
            &delta,
            &symbols,
            Default::default(),
        )
        .unwrap();
        let basis =
            VisibilitySnapshotBasis::from_applied_root(BranchId("main".into()), VersionId(1), root);
        Self {
            basis: ResidentVisibilitySnapshotBasis::admit(basis, execution, &symbols).unwrap(),
            instrumentation: RuntimeInstrumentation::new(),
            source: graph.source,
            target: graph.target,
            relation: graph.relation,
        }
    }

    fn walk(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedStructuralWalkScope<'_>,
    ) -> Result<PinnedStructuralWalk, PinnedStructuralWalkDenial> {
        complete(
            execution,
            scope,
            PinnedProjectionSource {
                resident: &self.basis,
                runtime_identity: 1,
                current_version: self.basis.version_id(),
                instrumentation: &self.instrumentation,
            },
        )
    }
}
