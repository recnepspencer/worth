//! Producer heads recorded through the lineage's restoration door (public
//! #429). No commit publishes such a head, so no settlement registration
//! wakes a reader of its family; only re-resolution can see it.
use super::{Fact, FactMovement};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use std::sync::OnceLock;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryCheckpointOutputRole,
};
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::SealedNativeOutputWitness;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::restoration::RestoredRecord;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::CertificationBasis;
use worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_runtime_world::facade::ProductBranchObservation;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::domain_computation::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, resolved_account,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;

/// A restored row's retained facts: the producer read only its scope entity.
pub(super) fn entity_facts(entity: EntityId) -> RetainedSourceFacts {
    retained_facts(vec![WorthQueryApplicationObservedFact::SourceEntity {
        entity_id: entity,
    }])
}

pub(super) fn retained_facts(facts: Vec<WorthQueryApplicationObservedFact>) -> RetainedSourceFacts {
    ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::from(facts))
}

/// A producer's checkpoint correspondence under `binding`: its one
/// `retained-account` role preserves `entity`.
pub(super) fn checkpoint_correspondence(
    binding: TypeId,
    entity: EntityId,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationOutputCorrespondence {
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        binding,
        binding,
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole::new_for_test(
            "retained-account".to_owned(),
            ApplicationMutationOutputPosture::Preserve,
            "Account".to_owned(),
            entity,
        )],
        |_| Some(TypeId::of_for_test::<Account>()),
        execution,
    )
    .unwrap()
}

/// A producer's checkpoint facts: its scope entity, and `output`'s kind and
/// every native output aspect revision at `snapshot`, which a restored-root
/// reader rebuilds the head's witness from.
pub(super) fn checkpoint_facts(
    world: &AuthorizationWorld,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    scope: EntityId,
    output: EntityId,
) -> Vec<WorthQueryApplicationObservedFact> {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let layout = graph.layout_for_test();
    let truth = runtime.read_truth();
    let mut facts = vec![
        WorthQueryApplicationObservedFact::SourceEntity { entity_id: scope },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: output,
            kind: truth
                .read_exact_snapshot_live_entity_kind(execution, snapshot, output)
                .unwrap()
                .unwrap(),
        },
    ];
    for aspect in layout.native_output_aspects_for_test("Account") {
        facts.push(WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id: output,
            aspect: aspect.clone(),
            native_revision: truth
                .read_exact_snapshot_entity_aspect_version(execution, snapshot, output, aspect)
                .unwrap()
                .unwrap(),
        });
    }
    facts
}

/// The account resolved by its `status` at the selected head.
pub(super) fn account(world: &AuthorizationWorld, status: &str) -> EntityId {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase.test_host_request_for_test())
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    status.to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call")
}

/// Delete the account whose status is `status` through a committed
/// application operation authorized on the account whose status is `on`,
/// which the principal owns, so the selected head no longer holds it.
pub(super) fn retire(world: &AuthorizationWorld, on: &str, status: &str) {
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let request = live_scope();
            let principal = authenticated_principal(&host, world, &request);
            let account = resolved_account(&host, world, on, &request);
            let retired = resolved_account(&host, world, status, &request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            let admitted = world
                .selected_product(&host)
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admitted,
                    |reader, _| {
                        let retired = reader
                            .resolve_entity(AccountStatus::at("AccountStatus"), status.to_owned())
                            .unwrap();
                        reader
                            .require_decision_entity(&retired, Account::reference())
                            .unwrap();
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let mut effects = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admitted,
                    projection,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let retired = effects.existing_entity(&retired).unwrap();
            effects
                .delete_entity(Account::reference(), &retired)
                .unwrap();
            effects.finish().unwrap()
        })
        .expect("the retirement owns its host call");
    let outcome = world.application.compare_and_commit_application(
        program,
        WorthQueryApplicationIdempotencyBinding::new_for_test([201; 32], [202; 32]),
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the retirement commits: {outcome:?}"
    );
}

/// The producer a head is recorded for: the family's scope account at the
/// selected head's observation.
pub(super) struct Producer<'w> {
    pub world: &'w AuthorizationWorld,
    pub entity: EntityId,
    authority: u64,
    schema: ApplicationSchemaBindingIdentity,
    scope: WorthQueryOperationScopeEntityBinding,
    observation: ProductBranchObservation,
}

impl<'w> Producer<'w> {
    /// The producer scoped to the account whose status is `status`.
    pub fn of(world: &'w AuthorizationWorld, status: &str) -> Self {
        let entity = account(world, status);
        let selected = world
            .application
            .with_host_advancement(|phase| {
                world.selected_product(&phase.test_host_request_for_test())
            })
            .expect("the fixture read owns its installed host call");
        let (_, product, _) = selected.into_parts_for_test();
        Self {
            world,
            entity,
            authority: (*world.application.runtime_for_test())
                .authority_identity_for_test()
                .as_u64(),
            schema: (*world.application.installed_schema_for_test()).binding_identity(),
            scope: WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity),
            observation: product.observation_for_test().clone(),
        }
    }

    /// Run `read` on the world's runtime with a serial request.
    pub fn with_runtime<T>(
        &self,
        read: impl FnOnce(&worth_execution::ExecutionRequest<'_, '_>, &RelationalRuntime) -> T,
    ) -> T {
        let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        graph
            .integration_handle_for_test()
            .with_runtime_for_test(execution, |runtime| read(execution, runtime))
    }

    /// `output`'s checkpoint facts under this producer's scope at the
    /// selected head's snapshot, where reads and certification look.
    pub fn checkpoint_facts(&self, output: EntityId) -> Vec<WorthQueryApplicationObservedFact> {
        let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let serial = worth_execution::ExecutionRequest::serial(&host_request);
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        self.world
            .application
            .with_host_advancement(|phase| {
                let selected = self
                    .world
                    .selected_product(&phase.test_host_request_for_test());
                graph
                    .integration_handle_for_test()
                    .with_runtime(&serial, |runtime| {
                        let snapshot = selected.application_basis_for_test().snapshot_handle();
                        checkpoint_facts(
                            self.world,
                            &serial,
                            runtime,
                            snapshot,
                            self.entity,
                            output,
                        )
                    })
            })
            .expect("the fixture read owns its installed host call")
    }

    /// `output`'s checkpoint facts as a checkpoint taken at another revision
    /// of it would carry them: every present output aspect one revision on.
    /// A head sealed from them is stale at the selected head.
    pub fn stale_checkpoint_facts(
        &self,
        output: EntityId,
    ) -> Vec<WorthQueryApplicationObservedFact> {
        self.checkpoint_facts(output)
            .into_iter()
            .map(|fact| match fact {
                WorthQueryApplicationObservedFact::SourceAspectRevision {
                    entity_id,
                    aspect,
                    native_revision,
                } => WorthQueryApplicationObservedFact::SourceAspectRevision {
                    entity_id,
                    aspect,
                    native_revision: native_revision.map(|revision| revision + 1),
                },
                fact => fact,
            })
            .collect()
    }

    /// The sealed witness `facts` prove for `correspondence`.
    pub fn witness(
        &self,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        facts: &[WorthQueryApplicationObservedFact],
    ) -> Arc<OnceLock<SealedNativeOutputWitness>> {
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        SealedNativeOutputWitness::from_checkpoint_facts_for_test(
            correspondence,
            graph.layout_for_test(),
            facts,
        )
        .unwrap()
        .expect("the head's checkpoint facts cover its output")
    }

    /// Record a head of `binding` for `output` in `partition`, carrying
    /// `facts`; `witnessed` gives it the sealed witness they prove, as a
    /// republication does, and otherwise it is an unwitnessed checkpoint
    /// restore root.
    pub fn record(
        &self,
        binding: TypeId,
        partition: [u8; 32],
        output: EntityId,
        facts: Vec<WorthQueryApplicationObservedFact>,
        witnessed: bool,
    ) -> RestoredRecord {
        let correspondence =
            self.with_runtime(|execution, _| checkpoint_correspondence(binding, output, execution));
        let witness = witnessed.then(|| self.witness(&correspondence, &facts));
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        graph
            .integration_handle_for_test()
            .output_lineage
            .lock()
            .unwrap()
            .record_restoration_for_test(
                binding,
                self.authority,
                self.schema.clone(),
                self.scope,
                &self.observation,
                Arc::new(correspondence),
                RecordedSourceIdentity::Checkpoint(
                    WorthQueryCheckpointSourceIdentity::new_for_test([partition[0] ^ 0x01; 32]),
                ),
                partition,
                None,
                [partition[0] ^ 0x02; 32],
                retained_facts(facts),
                witness,
                None,
                None,
            )
            .expect("the producer's head is recorded")
    }

    /// Whether the recorded row `head` still has no witness.
    pub fn unwitnessed(&self, head: &RestoredRecord) -> bool {
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        let handle = graph.integration_handle_for_test();
        let lineage = handle.output_lineage.lock().unwrap();
        let coordinate = head.identity.coordinate();
        lineage
            .by_source_for_test()
            .get(head.identity.source())
            .and_then(|occurrences| occurrences.get(coordinate.occurrence_for_test()))
            .and_then(|history| history.get(coordinate.generation_for_test()))
            .and_then(|records| records.get(head.identity.slot()))
            .and_then(|cell| cell.get())
            .expect("the head is recorded")
            .native_output_witness_for_test()
            .get()
            .is_none_or(|witness| witness.get().is_none())
    }

    /// Every recorded partition but `except`, as an overlay masks them.
    pub fn partitions_except(&self, except: [u8; 32]) -> Vec<(Option<TypeId>, [u8; 32])> {
        let graph = (*self.world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        let handle = graph.integration_handle_for_test();
        let lineage = handle.output_lineage.lock().unwrap();
        let mut partitions: Vec<_> = lineage
            .by_source_for_test()
            .values()
            .flat_map(|occurrences| occurrences.values())
            .flat_map(|history| history.values())
            .flatten()
            .filter_map(|cell| {
                cell.get()
                    .and_then(|recorded| *recorded.source_partition_identity_for_test())
            })
            .filter(|partition| *partition != except)
            .map(|partition| (None, partition))
            .collect();
        partitions.sort_by_key(|(_, partition)| *partition);
        partitions.dedup_by_key(|(_, partition)| *partition);
        partitions
    }
}

/// Certify `fact` at the selected head's basis through the site 2 rebase
/// door (`postcommit_currentness_at`), which re-applies the read's overlay.
pub(super) fn certify_rebased(world: &AuthorizationWorld, fact: &Fact) -> FactMovement {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let serial = worth_execution::ExecutionRequest::serial(&host_request);
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let selected = world.selected_product(&host);
            graph
                .integration_handle_for_test()
                .with_runtime(&serial, |runtime| {
                    let snapshot = selected.application_basis_for_test().snapshot_handle();
                    let product = selected.product();
                    let basis = CertificationBasis::of(
                        product.lifecycle_incarnation(),
                        product.reference_generation(),
                        snapshot,
                    );
                    fact.postcommit_currentness_at(
                        runtime,
                        &host.borrowed_request_for_test(),
                        basis,
                    )
                    .expect("certification is not stopped")
                    .expect("certification is not refused")
                })
        })
        .expect("the fixture certification owns its installed host call")
}

/// `fact` as if its read's own commit had applied `applied` and masked
/// `replaced`: the shape a site 2 rebase certifies.
pub(super) fn overlaid(
    fact: &Fact,
    applied: &[EntityId],
    replaced: &[(Option<TypeId>, [u8; 32])],
) -> Fact {
    let Fact::FamilySelection { observation } = fact else {
        panic!("only a FamilySelection fact carries an overlay: {fact:?}");
    };
    Fact::FamilySelection {
        observation: Arc::new(observation.with_overlay_for_test(applied.iter().copied(), replaced)),
    }
}
