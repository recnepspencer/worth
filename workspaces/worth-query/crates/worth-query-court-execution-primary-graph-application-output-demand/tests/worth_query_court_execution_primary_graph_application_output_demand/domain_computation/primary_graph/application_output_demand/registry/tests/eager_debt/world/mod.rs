//! Real publication and scope provenance; plain shapes are shared with the model.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;
use worth_runtime_world::facade::CompositeCommitIdentity;
pub struct Context {
    pub source: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource,
    pub completed: WorthQueryApplicationCommitReceipt,
    pub scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
    pub ready: DemandState,
}
impl Context {
    pub fn new() -> Self {
        let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let mut receipt =
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::commit_on_world_with_output_demand_observation(
                &world,
                221,
                "open",
                "eager-bind",
            );
        let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation_for_test()
            .unwrap();
        let source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource::new_for_test(receipt, change, observation, None);
        Self {
            source,
            completed: worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::commit_on_world_with_output_demand_observation(
                &world,
                222,
                "eager-bind",
                "eager-newer",
            ),
            scope: worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence().0,
            ready: worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(),
        }
    }
    pub fn commit(&self, id: usize) -> CompositeCommitIdentity {
        if id == 0 {
            (*self.source.change_for_test()).product_commit().clone()
        } else {
            self.completed
                .committed_product_publication()
                .composite_commit()
                .clone()
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
pub struct World<'a> {
    pub context: &'a Context,
    pub registry: WorthQueryOutputDemandRegistry,
    pub model: Model,
    pub keys: Vec<WorthQueryOutputDemandKey>,
    pub owners: Vec<Option<WorthQueryOutputDemandInterest>>,
    pub generation: u64,
    pub failure: Option<String>,
    pub extra_dependent: Option<usize>,
}
impl<'a> World<'a> {
    pub fn new(context: &'a Context, seed: u64) -> Self {
        let generation = seed + 10;
        let late = seed % 4 >= 2;
        let mut model = Model::default();
        let mut keys = vec![
            key_with_identity("oracle-upstream", generation, 1, 80),
            key_with_identity("oracle-upstream", generation + 1, 1, 81),
            key("oracle-dependent", generation, 2),
        ];
        let old_owner = usize::from(seed & 8 != 0);
        for (id, row) in [
            Row {
                occurrence: 0,
                product: 0,
                generation,
                owners: old_owner,
                phase: if late {
                    Phase::Ready
                } else {
                    Phase::Superseded
                },
                claims: vec![],
                claim_slots: 0,
                source: false,
            },
            Row {
                occurrence: 0,
                product: 0,
                generation: generation + 1,
                owners: 1,
                phase: if late { Phase::Running } else { Phase::Ready },
                claims: vec![],
                claim_slots: 0,
                source: true,
            },
            Row {
                occurrence: 1,
                product: 1,
                generation,
                owners: 1,
                phase: Phase::Ready,
                claims: vec![0],
                claim_slots: 1,
                source: false,
            },
        ]
        .into_iter()
        .enumerate()
        {
            model.rows.insert(id, row);
        }
        if seed % 3 == 2 {
            let id = keys.len();
            keys.push(key_with_identity("oracle-upstream", generation - 1, 1, 79));
            model.rows.insert(
                id,
                Row {
                    occurrence: 0,
                    product: 0,
                    generation: generation - 1,
                    owners: 0,
                    phase: if late {
                        Phase::Ready
                    } else {
                        Phase::Superseded
                    },
                    claims: vec![],
                    claim_slots: 0,
                    source: false,
                },
            );
            model.rows.get_mut(&2).unwrap().claims.push(id);
            model.rows.get_mut(&2).unwrap().claim_slots = 2;
        }
        let extra_dependent = (seed & 16 != 0).then(|| {
            let id = keys.len();
            keys.push(key("oracle-second-dependent", generation, 3));
            model.rows.insert(
                id,
                Row {
                    occurrence: 2,
                    product: 1,
                    generation,
                    owners: 1,
                    phase: Phase::Ready,
                    claims: vec![0],
                    claim_slots: 1,
                    source: false,
                },
            );
            id
        });
        for neighbor in 0..seed % 3 {
            let id = keys.len();
            keys.push(key("oracle-foreign", generation, 220 + neighbor as u8));
            model.rows.insert(
                id,
                Row {
                    occurrence: 3 + neighbor as usize,
                    product: 0,
                    generation,
                    owners: 1,
                    phase: Phase::Pending,
                    claims: vec![],
                    claim_slots: 0,
                    source: false,
                },
            );
        }
        model.custody.insert(
            0,
            Custody {
                product: 0,
                tokens: 1,
                completed: false,
                bound: None,
                consumed: false,
                denied: false,
                retired: false,
            },
        );
        model.custody.insert(
            1,
            Custody {
                product: 0,
                tokens: usize::from(seed & 4 != 0),
                completed: true,
                bound: Some(generation + 1),
                consumed: true,
                denied: false,
                retired: false,
            },
        );
        let registry = WorthQueryOutputDemandRegistry::default();
        let mut state = (*registry.state_for_test()).lock().unwrap();
        let mut owners = Vec::new();
        for (&id, row) in &model.rows {
            let phase = match row.phase {
                Phase::Ready => copy_ready(&context.ready),
                Phase::Superseded => copy_ready(&context.ready),
                _ => DemandState::Admitted,
            };
            let occurrence = if row.product == 0 {
                context.completed.product_branch().occurrence_for_test()
            } else {
                occurrence()
            };
            let mut record = record(occurrence, phase, row.owners);
            (*record.framework_required_count_mut_for_test()) = model.claimed(id);
            (*record.prerequisites_mut_for_test()) = row
                .claims
                .iter()
                .map(|&up| Arc::new(keys[up].clone()))
                .collect();

            if row.source {
                (*record.source_commits_mut_for_test()).push(context.commit(1));
            }
            let wake = Arc::clone(record.wake_for_test());
            (*state.records_mut_for_test()).insert_for_test(keys[id].clone(), record);
            let member = state.prepare_required_member_for_test(&keys[id]).unwrap();
            state.install_required_member_for_test(member);
            retain_owned_key(&mut state, &keys[id]);
            owners.push((row.owners != 0).then(|| interest(&registry, keys[id].clone(), wake)));
        }
        for (&id, row) in &model.rows {
            if !row.claims.is_empty() {
                let upstream =
                    (*(*state.records_for_test())[&keys[id]].prerequisites_for_test()).clone();
                let batch = state
                    .prepare_claim_readthrough_for_test(&keys[id], &upstream)
                    .unwrap();
                if let Some(batch) = &batch {
                    batch.install_for_test();
                    batch.publish_for_test();
                }
                (*(*state.records_mut_for_test())
                    .get_mut(&keys[id])
                    .unwrap()
                    .claim_batch_mut_for_test()) = batch;
            }
        }
        for (&id, entry) in &model.custody {
            let identity = (*keys[1].source_for_test()).clone();
            state.source_custody.insert(context.commit(id), {
                // Bound in the original literal order, passed in the door's.
                let (
                    occurrence,
                    root_kind,
                    source,
                    discovery,
                    bound_sources,
                    consumed_sources,
                    retired_sources,
                    retired,
                    token_count,
                    cleanup,
                    completed,
                ) = (
                    context.completed.product_branch().occurrence_for_test(),
                    PreparedOutputRootKind::Discovered(EqualityTypeId::of_for_test::<()>()),
                    (id == 0).then(|| context.source.clone()),
                    None,
                    entry.bound.map(|_| {
                        vec![BoundOutputSource::new_for_test(
                            context.scope,
                            identity.clone(),
                        )]
                    }),
                    if entry.consumed {
                        vec![identity]
                    } else {
                        vec![]
                    },
                    vec![],
                    None,
                    entry.tokens,
                    Default::default(),
                    entry.completed,
                );
                SourceCustody::new_for_test(
                    occurrence,
                    root_kind,
                    source,
                    discovery,
                    consumed_sources,
                    retired_sources,
                    bound_sources,
                    retired,
                    token_count,
                    cleanup,
                    completed,
                )
            });
        }
        if !late {
            // The claimed predecessor really became superseded under an admission.
            supersede_predecessors(&mut state, &keys[1]).unwrap();
        }
        drop(state);
        registry.complete_prepared_source_for_test(&context.completed);
        Self {
            context,
            registry,
            model,
            keys,
            owners,
            generation,
            failure: None,
            extra_dependent,
        }
    }
    pub fn drop_owner(&mut self, id: usize) {
        if self.owners[id].is_some() {
            drop(self.owners[id].take());
            self.model.drop_owner(id);
        }
    }
    pub fn admission(&self) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut interest = self
            .registry
            .admit_for_test(
                key("oracle-reader", self.generation, 250),
                None,
                self.context.scope,
                occurrence(),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
                None,
                None,
            )
            .map(|(interest, _pause)| interest)?;
        // This request row is unrelated to the compared domain; release through real Drop.
        (*(*(*self.registry.state_for_test())
            .lock()
            .unwrap()
            .records_mut_for_test())
        .get_mut(interest.key_for_test())
        .unwrap()
        .state_mut_for_test()) = DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "oracle request completed",
        ));
        (*interest.requires_output_mut_for_test()) = false;
        drop(interest);
        Ok(())
    }
    pub fn bind(&mut self) {
        self.registry
            .ensure_discovered_sources_bound_for_test(
                &self.context.commit(0),
                &[BoundOutputSource::new_for_test(
                    self.context.scope,
                    (*self.keys[0].source_for_test()).clone(),
                )],
            )
            .unwrap();
        self.model.bind(0, self.generation);
    }
}
pub fn copy_ready(ready: &DemandState) -> DemandState {
    let DemandState::Output(output) = ready else {
        panic!("Ready fixture")
    };
    let Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(completion)) =
        output.checkpoint_for_test()
    else {
        panic!("Ready fixture")
    };
    DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::new_for_test(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(completion.clone()),
    ))
}
