use super::observation::ProductObserver;
use super::*;
use worth_relational::facade::history::BranchId;
use worth_signal::facade::branch::validate_signal_branch_name;

pub(super) struct ModelRun {
    pub court: CompositeSupplyChainCourt,
    pub model: ProductModel,
    pub heads: BTreeMap<String, ProductBranchObservation>,
    pub held: BTreeMap<usize, ProductBranchObservation>,
    pub observer: ProductObserver,
    pub steps: Vec<String>,
}
pub(in super::super) fn run(seed: u64, rounds: usize) {
    let court = CompositeSupplyChainCourt::compile();
    let model = ProductModel::bootstrap();
    let root = court.bootstrap();
    let mut run = ModelRun {
        court,
        model,
        heads: BTreeMap::from([("main".into(), root)]),
        held: BTreeMap::new(),
        observer: ProductObserver::default(),
        steps: vec!["bootstrap".into()],
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run.check();
        for (index, (relational, signal)) in
            [(false, false), (true, false), (false, true), (true, true)]
                .into_iter()
                .enumerate()
        {
            run.create(index, relational, signal);
            run.retire_and_reclaim();
        }
        let mut random = seed;
        for round in 0..rounds {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            run.create(round + 4, true, true);
            // Vary legal boundary adjacencies, while every round still visits
            // all publication postures, cancellation, staleness and recovery.
            let early_hold = random & 1 == 0;
            if early_hold {
                run.hold();
            }
            let amount = 1 + random % 11;
            if random & 2 == 0 {
                run.publish(Some(amount), false);
                if !early_hold {
                    run.hold();
                }
                run.publish(None, true);
            } else {
                run.publish(None, true);
                if !early_hold {
                    run.hold();
                }
                run.publish(Some(amount), false);
            }
            run.publish(Some(1 + (random >> 8) % 11), true);
            if random & 4 == 0 {
                run.release();
            }
            if random & 8 == 0 {
                run.cancel();
                run.stale();
            } else {
                run.stale();
                run.cancel();
            }
            if random & 4 != 0 {
                run.release();
            }
            run.partial_cleanup();
            run.retire_and_reclaim();
        }
    }));
    if let Err(payload) = result {
        eprintln!(
            "seed={seed:#x}; shortest checked failing prefix ({} transitions): {:?}",
            run.steps.len(),
            run.steps
        );
        std::panic::resume_unwind(payload);
    }
    run.heads.clear();
    run.court.finish();
}
impl ModelRun {
    fn hold(&mut self) {
        self.steps.push("observe/retain".into());
        self.model.hold(0, "work");
        self.held.insert(0, self.court.observe(&self.heads["work"]));
        self.check();
    }
    fn release(&mut self) {
        self.steps.push("release observation".into());
        self.model.release(0);
        self.held.remove(&0);
        self.check();
    }
    pub fn check(&mut self) {
        self.observer
            .check(&self.court, &self.model, &self.heads, &self.held);
    }
    fn create(&mut self, index: usize, relational: bool, signal: bool) {
        self.steps
            .push(format!("create work/{relational}/{signal}/{index}"));
        self.model.create("work", relational, signal);
        let target = format!("model-component-{index}");
        let plans = ProductBranchCreationPlans::new(
            if relational {
                RelationalBranchCreationPlan::ForkExact {
                    target: BranchId(target.clone()),
                }
            } else {
                RelationalBranchCreationPlan::ReuseExact
            },
            if signal {
                SignalBranchCreationPlan::ForkExact {
                    target: validate_signal_branch_name(target).unwrap(),
                }
            } else {
                SignalBranchCreationPlan::ReuseExact
            },
        );
        let RuntimeWorldBranchCreationOutcome::Performed(head) = self
            .court
            .world
            .branch_port()
            .create_product_branch(
                self.heads["main"].clone(),
                ProductBranchCreationIntent::from_source("work", plans).unwrap(),
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("model predicts a healthy branch creation");
        };
        self.heads.insert("work".into(), head);
        self.check();
    }
    fn retire_and_reclaim(&mut self) {
        self.steps.push("retire work".into());
        let head = self.heads.remove("work").unwrap();
        let chain = self.model.ancestry(self.model.branches["work"].head);
        let trace = self
            .court
            .world
            .inspection_port()
            .trace_ancestry(
                head.selected_commit().clone(),
                std::num::NonZeroUsize::new(128).unwrap(),
            )
            .unwrap();
        let keys: Vec<_> = trace
            .commits()
            .flat_map(|c| {
                [
                    RuntimeWorldRetentionKey::relational(c.basis()),
                    RuntimeWorldRetentionKey::signal(c.basis()),
                ]
            })
            .collect();
        drop(trace);
        self.model.retire("work");
        drop(
            self.court
                .world
                .branch_port()
                .retire_product_branch(&head)
                .unwrap(),
        );
        drop(head);
        self.check();
        for identity in chain {
            self.steps.push(format!("reclaim {identity}"));
            let expected = self.model.reclaim(identity);
            let commit = self.observer.commit(identity);
            let report = self
                .court
                .world
                .lifecycle_port()
                .reclaim_history(CompositeHistoryReclamationRequest::new(
                    self.court.world.owner_identity(),
                    vec![commit.clone()],
                    1,
                ))
                .unwrap();
            assert!(report.examined() <= 1);
            assert_eq!(
                report.reclaimed_commits(),
                if expected {
                    std::slice::from_ref(&commit)
                } else {
                    &[]
                }
            );
            self.check();
        }
        for keys in keys.chunks(2) {
            let report = self
                .court
                .world
                .lifecycle_port()
                .reclaim_retention(keys, 2)
                .unwrap();
            assert!(report.examined() <= 2);
        }
    }
}
