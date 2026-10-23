//! The installed poll-only owner retains actual facts and canonical state without a bill.
use super::super::tree_report::WorthQueryPartitionedTreeRun as TreeRun;
use super::*;
use worth_foundational::facade::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
mod selective;

struct PollOnly(Arc<Owner>);
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for PollOnly {
    type Operation = TouchAccountOperation;
    type Item = Number;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Number>, WorthQueryComputationInputDenial<u32>>
    {
        self.0.partitions(reader, input)
    }
    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        self.0.partition_key(reader, input, item)
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.0.gathered.lock().unwrap().push(partition.key().0);
        let sum = partition.items().map(|(_, item)| item.0).sum::<u64>();
        let extra = if partition.key().0 == 1 {
            reader
                .field(input, AccountLabel::reference())?
                .expect("seeded label")
                .len()
        } else {
            0
        };
        Ok(sum + u64::try_from(extra).unwrap())
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        self.0.calls.lock().unwrap()[3] += 1;
        checkpoint.poll()?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        self.0.reducer()
    }
    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}
impl TestOwner for PollOnly {
    fn gathered(&self) -> &Mutex<Vec<u64>> {
        &self.0.gathered
    }
}
type NativeInstalled =
    WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, PollOnly>;

struct NativeAttempt {
    value: u64,
    sealed: SealedComputationRun,
    runs: Vec<(Run, Option<ExecutionReport>)>,
    trees: Vec<super::super::tree_report::WorthQueryPartitionedTreeRun>,
    gathered: Vec<u64>,
}

fn policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    )
}

fn native_attempt<Owned: TestOwner>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owned>,
    prior: ComputationPrior,
) -> NativeAttempt {
    let request = live_scope();
    let mut observed = None;
    world
        .application
        .with_application_advancement(&request, |phase| {
            let phase_custody = phase.test_host_request();
            let admission = projected_attempt::admit(world, &request, &phase);
            let execution = phase
                .execution_for(&world.application.product_runtime)
                .unwrap();
            assert!(!execution.has_accounting());
            let allocation = execution.allocation_policy().unwrap();
            runs();
            tree_runs();
            installed.owner.gathered().lock().unwrap().clear();
            ComputationPrior::hand_in_test(Some(prior));
            SealedComputationRun::keep_in_test(None);
            let mut value = None;
            let completed = world
                .invariant
                .project_admitted_operation(
                    &phase_custody.phase_for_application(&world.application),
                    &admission,
                    |reader, root| {
                        let (computed, _) = execution
                            .execution_request()
                            .unwrap()
                            .run_native(|_| {
                                installed
                                    .prepare_through(reader, execution, root)
                                    .unwrap()
                                    .compute(WorthQueryManagedComputationExecution::new(execution))
                            })
                            .unwrap();
                        let computed = computed.unwrap();
                        assert_eq!(computed.accounted_work(), None);
                        value = Some(computed.complete().unwrap());
                    },
                    allocation,
                )
                .unwrap();
            let (_, projection, _) = completed.into_parts();
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase_custody.phase_for_application(&world.application),
                    admission,
                    projection,
                    allocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase_custody.phase_for_application(&world.application),
                    allocation,
                )
                .unwrap();
            let sealed =
                SealedComputationRun::kept_in_test().expect("native producer retains its state");
            assert!(
                sealed.state.facts.facts().next().is_some(),
                "real reads remain retained"
            );
            assert_eq!(sealed.state.facts.observation_work_bound(), None);
            let typed = sealed
                .state
                .typed
                .downcast_ref::<super::super::retained::RetainedPartitions<Parity, Number, u64>>()
                .unwrap();
            assert!(sealed.attribution_memory.is_none());
            assert_eq!(typed.call_sequence.bytes(), None);
            assert_eq!(typed.partitions.bytes(), None);
            execution
                .with_native_lease(|lease| {
                    assert_eq!(
                        lease.charged_memory_bytes(),
                        0,
                        "native retained metadata opens no byte ledger"
                    );
                })
                .unwrap();
            assert!(typed.membership.declared_units.is_none());
            assert!(typed
                .item_keys
                .values()
                .all(|call| call.declared_units.is_none()));
            assert_eq!(typed.items.len(), typed.digests.len());
            assert_eq!(typed.routing.partitions().count(), 2);
            assert_eq!(
                typed.call_sequence.prefix(),
                typed.items.len() + 1,
                "membership plus stable item keys"
            );
            assert_eq!(
                typed.call_sequence.shape().len(),
                typed.items.len() + 1 + typed.partitions.len(),
                "one gather per canonical partition"
            );
            assert_eq!(typed.native_run.accounted_cost(), None);
            assert_eq!(*typed.tree.result(), value.unwrap());
            let mut gathered = installed.owner.gathered().lock().unwrap().clone();
            gathered.sort_unstable();
            observed = Some(NativeAttempt {
                value: value.unwrap(),
                sealed,
                runs: runs(),
                trees: tree_runs(),
                gathered,
            });
            ComputationPrior::hand_in_test(None);
        })
        .unwrap();
    observed.unwrap()
}

fn retained_prior(sealed: SealedComputationRun) -> ComputationPrior {
    ComputationPrior::new(edition(), Ok(Arc::new(sealed.state)), None)
}

#[test]
fn worker_only_partition_run_retains_facts_reuses_and_observes_a_real_source_edit() {
    use worth_query_declaration::facade::application_schema::{
        ApplicationScalarValueBinding, StringApplicationValueBinding,
    };
    use worth_relational::facade::transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    };
    let _observation = QueryObservation::install_on_this_thread();
    let world = crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources(
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources()
            .with_execution_policy(policy()),
    );
    let base = installed(StatusRead::Gather(1), sum);
    let installed = NativeInstalled::new(
        PollOnly(Arc::clone(&base.owner)),
        ComputationRetention::ProducerOperation,
    );
    let first = native_attempt(
        &world,
        &installed,
        ComputationPrior::new(edition(), Err(Cause::FirstRun), None),
    );
    assert_eq!(first.value, 10 + "primary".len() as u64);
    assert_eq!(first.gathered, [0, 1]);
    assert!(
        matches!(first.runs.as_slice(), [(Run::Full(Cause::FirstRun), Some(report))]
        if !report.has_accounting())
    );
    assert!(first
        .trees
        .iter()
        .all(|tree| tree.metrics_for_test().accounted_metrics().is_none()));
    assert!(matches!(
        first.trees.as_slice(),
        [TreeRun::Full(Cause::FirstRun, _)]
    ));
    let calls = *base.owner.calls.lock().unwrap();
    let unchanged = native_attempt(&world, &installed, retained_prior(first.sealed));
    assert_eq!(unchanged.value, first.value);
    assert!(unchanged.gathered.is_empty());
    assert_eq!(
        *base.owner.calls.lock().unwrap(),
        calls,
        "all actual owner calls carry"
    );
    assert!(matches!(
        unchanged.runs.as_slice(),
        [(Run::Incremental, None)]
    ));
    let subject = unchanged
        .sealed
        .state
        .facts
        .facts()
        .find_map(|(_, fact, _)| {
            if let WorthQueryApplicationObservedFact::Field { entity_id, .. } = fact {
                Some(*entity_id)
            } else {
                None
            }
        })
        .unwrap();
    let field = AccountLabel::reference();
    let locator = world
        .application
        .primary_provider
        .graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let label = "worker-only-updated";
    let request = live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .execution_for(&world.application.product_runtime)
                .unwrap();
            assert!(!execution.has_accounting());
            let execution = execution.execution_request().unwrap();
            crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation(
                &execution,
                &world,
                WorkerIntentBatch::new("worker-only-partition-source-edit").push(
                    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: subject,
                            fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                                locator,
                                StringApplicationValueBinding::encode(&label.to_owned()).unwrap(),
                            )])),
                        },
                    )),
                ),
            );
        })
        .unwrap();
    let edited = native_attempt(&world, &installed, retained_prior(unchanged.sealed));
    assert_eq!(edited.value, 10 + label.len() as u64);
    assert_eq!(
        edited.gathered,
        [1],
        "only the partition consuming the edited field gathers again"
    );
    assert!(matches!(edited.runs.as_slice(), [(Run::Incremental, None)]));
    assert!(edited
        .trees
        .iter()
        .all(|tree| tree.metrics_for_test().accounted_metrics().is_none()));
    assert!(matches!(edited.trees.as_slice(), [TreeRun::Edited(_)]));
    let fresh = native_attempt(
        &world,
        &installed,
        ComputationPrior::new(edition(), Err(Cause::FirstRun), None),
    );
    assert_eq!(
        edited.value, fresh.value,
        "independent full native replay agrees"
    );
}
