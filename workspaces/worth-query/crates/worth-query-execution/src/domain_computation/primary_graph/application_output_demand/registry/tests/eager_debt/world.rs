//! Real publication and scope provenance; plain shapes are shared with the model.
use super::super::super::{BoundOutputSource, PreparedOutputRootKind, SourceCustody};
use super::*;
use crate::domain_computation::primary_graph::tests::{fixture, recoverable_commit_support};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::equality_type_id::EqualityTypeId;
use worth_runtime_world::facade::CompositeCommitIdentity;
pub(super) struct Context {
    pub source: super::super::super::WorthQueryPerformedOutputDemandSource,
    pub completed: WorthQueryApplicationCommitReceipt,
    pub scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub ready: DemandState,
}
impl Context {
    pub fn new() -> Self {
        let world = fixture::installed_authorization_world(true);
        let mut receipt =
            recoverable_commit_support::commit_on_world_with_output_demand_observation(
                &world,
                221,
                "open",
                "eager-bind",
            );
        let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation()
            .unwrap();
        let source = super::super::super::WorthQueryPerformedOutputDemandSource {
            receipt,
            change,
            observation,
            output_source_identity: None,
        };
        Self {
            source,
            completed: recoverable_commit_support::commit_on_world_with_output_demand_observation(
                &world,
                222,
                "eager-bind",
                "eager-newer",
            ),
            scope: source_custody::real_scope_and_occurrence().0,
            ready: closed_retirement::ready(),
        }
    }
    pub fn commit(&self, id: usize) -> CompositeCommitIdentity {
        if id == 0 {
            self.source.change.product_commit().clone()
        } else {
            self.completed
                .committed_product_publication()
                .composite_commit()
                .clone()
        }
    }
}
pub(super) struct World<'a> {
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
        let mut state = registry.state.lock().unwrap();
        let mut owners = Vec::new();
        for (&id, row) in &model.rows {
            let phase = match row.phase {
                Phase::Ready => copy_ready(&context.ready),
                Phase::Superseded => copy_ready(&context.ready),
                _ => DemandState::Admitted,
            };
            let occurrence = if row.product == 0 {
                context.completed.product_branch().occurrence()
            } else {
                occurrence()
            };
            let mut record = record(occurrence, phase, row.owners);
            record.framework_required_count = model.claimed(id);
            record.prerequisites = row
                .claims
                .iter()
                .map(|&up| Arc::new(keys[up].clone()))
                .collect();
            state.required_reserved_bytes += record.prerequisites.capacity()
                * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            if row.source {
                record.source_commits.push(context.commit(1));
            }
            let wake = Arc::clone(&record.wake);
            state.records.insert(keys[id].clone(), record);
            let member = state.prepare_required_member(&keys[id]).unwrap();
            state.install_required_member(member);
            retain_owned_key(&mut state, &keys[id]);
            owners.push((row.owners != 0).then(|| interest(&registry, keys[id].clone(), wake)));
        }
        for (&id, row) in &model.rows {
            if !row.claims.is_empty() {
                let upstream = state.records[&keys[id]].prerequisites.clone();
                let batch = state
                    .prepare_claim_readthrough(&keys[id], &upstream, &mut record_admission())
                    .unwrap();
                if let Some(batch) = &batch {
                    batch.install();
                    batch.publish();
                }
                state.records.get_mut(&keys[id]).unwrap().claim_batch = batch;
            }
        }
        for (&id, entry) in &model.custody {
            let identity = keys[1].source.clone();
            state.source_custody.insert(
                context.commit(id),
                SourceCustody {
                    occurrence: context.completed.product_branch().occurrence(),
                    root_kind: PreparedOutputRootKind::Discovered(EqualityTypeId::of::<()>()),
                    source: (id == 0).then(|| context.source.clone()),
                    discovery: None,
                    bound_sources: entry.bound.map(|_| {
                        vec![BoundOutputSource {
                            scope: context.scope,
                            identity: identity.clone(),
                        }]
                    }),
                    consumed_sources: if entry.consumed {
                        vec![identity]
                    } else {
                        vec![]
                    },
                    retired_sources: vec![],
                    retired: None,
                    token_count: entry.tokens,
                    cleanup: Default::default(),
                    completed: entry.completed,
                },
            );
        }
        if !late {
            // The claimed predecessor really became superseded under an admission.
            supersede_predecessors(&mut state, &keys[1], &mut record_admission()).unwrap();
        }
        drop(state);
        registry.complete_prepared_source(&context.completed);
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
    pub fn admission(&self, ceiling: u64) -> Result<u64, WorthQueryOutputDemandDenial> {
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: Some(ceiling),
            maximum_preparation_bytes: u64::MAX,
        });
        let mut interest = self
            .registry
            .admit(
                key("oracle-reader", self.generation, 250),
                None,
                self.context.scope,
                occurrence(),
                super::super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
                &mut admission,
            )
            .map(|(interest, _pause)| interest)?;
        // This request row is unrelated to the compared domain; release through real Drop.
        self.registry
            .state
            .lock()
            .unwrap()
            .records
            .get_mut(&interest.key)
            .unwrap()
            .state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "oracle request completed",
        ));
        interest.requires_output = false;
        drop(interest);
        Ok(admission.charged_work())
    }
    pub fn own_work(context: &Context, generation: u64) -> u64 {
        let registry = WorthQueryOutputDemandRegistry::default();
        let mut admission = record_admission();
        let interest = registry
            .admit(
                key("oracle-reader", generation, 250),
                None,
                context.scope,
                occurrence(),
                super::super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
                &mut admission,
            )
            .map(|(interest, _pause)| interest)
            .unwrap();
        let work = admission.charged_work();
        drop(interest);
        work
    }
    pub fn bind(&mut self) {
        self.registry
            .ensure_discovered_sources_bound(
                &self.context.commit(0),
                &[BoundOutputSource {
                    scope: self.context.scope,
                    identity: self.keys[0].source.clone(),
                }],
            )
            .unwrap();
        self.model.bind(0, self.generation);
    }
}
fn copy_ready(ready: &DemandState) -> DemandState {
    let DemandState::Output(output) = ready else {
        panic!("Ready fixture")
    };
    let Some(super::super::super::WorthQueryOutputCheckpoint::Ready(completion)) =
        &output.checkpoint
    else {
        panic!("Ready fixture")
    };
    DemandState::Output(super::super::super::WorthQueryOutputProgress::new(
        super::super::super::WorthQueryOutputCheckpoint::Ready(completion.clone()),
    ))
}
