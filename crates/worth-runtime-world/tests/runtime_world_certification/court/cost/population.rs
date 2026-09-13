use super::*;
use worth_relational::facade::history::BranchId;
use worth_signal::facade::branch::validate_signal_branch_name;

#[derive(Debug, Clone, Copy)]
pub(super) enum Axis {
    B,
    H,
    U,
    A,
    P,
    O,
}
impl Axis {
    pub const ALL: [Self; 6] = [Self::B, Self::H, Self::U, Self::A, Self::P, Self::O];
}
pub(super) struct Population {
    pub court: CompositeSupplyChainCourt,
    pub root: ProductBranchObservation,
    branches: Vec<ProductBranchObservation>,
    observations: Vec<ProductBranchObservation>,
    attempts: Vec<PreparedCompositePublicationWithoutSignal>,
    partials: Vec<ProductUnpublishedOwnerEffects>,
}
pub(super) fn child(
    court: &CompositeSupplyChainCourt,
    root: &ProductBranchObservation,
    name: &str,
    relational: bool,
    signal: bool,
) -> ProductBranchObservation {
    let plans = ProductBranchCreationPlans::new(
        if relational {
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId(name.into()),
            }
        } else {
            RelationalBranchCreationPlan::ReuseExact
        },
        if signal {
            SignalBranchCreationPlan::ForkExact {
                target: validate_signal_branch_name(name).unwrap(),
            }
        } else {
            SignalBranchCreationPlan::ReuseExact
        },
    );
    let RuntimeWorldBranchCreationOutcome::Performed(head) = court
        .world
        .branch_port()
        .create_product_branch(
            root.clone(),
            ProductBranchCreationIntent::from_source(name, plans).unwrap(),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
    else {
        panic!("cost population creates real independent branch");
    };
    head
}
impl Population {
    pub fn build(axis: Axis, size: usize) -> Self {
        let court = CompositeSupplyChainCourt::compile();
        let root = court.bootstrap();
        let mut population = Self {
            court,
            root,
            branches: vec![],
            observations: vec![],
            attempts: vec![],
            partials: vec![],
        };
        for index in 1..size {
            let name = format!("population-{index}");
            match axis {
                Axis::B => population.branches.push(child(
                    &population.court,
                    &population.root,
                    &name,
                    false,
                    false,
                )),
                Axis::H => {
                    let receipt = population
                        .court
                        .publish_cargo(&population.root, &(index % 11 + 1).to_string());
                    population.root = population.court.observe(&population.root);
                    drop(receipt);
                }
                Axis::U => {
                    let head = child(&population.court, &population.root, &name, true, false);
                    drop(
                        population
                            .court
                            .world
                            .branch_port()
                            .retire_product_branch(&head)
                            .unwrap(),
                    );
                    drop(head);
                    // Retained history keeps each distinct exact Relational
                    // pin live. H co-varies with U; B returns to one.
                }
                Axis::A => population
                    .attempts
                    .push(population.court.prepare_cargo(&population.root, "5")),
                Axis::P => {
                    let head = child(&population.court, &population.root, &name, true, false);
                    population
                        .partials
                        .push(population.court.partial_after_relational(&head));
                    population.branches.push(head);
                }
                Axis::O => population
                    .observations
                    .push(population.court.observe(&population.root)),
            }
        }
        if matches!(axis, Axis::B | Axis::H | Axis::U | Axis::O) {
            assert_eq!(
                population
                    .court
                    .world
                    .inspection_port()
                    .retention_costs()
                    .unwrap()
                    .signal_contacts(),
                1,
                "only bootstrap acquires the reused Signal owner lease"
            );
        }
        let (b, h, u, a, p, o) = population.axes();
        match axis {
            Axis::B => assert_eq!(b, size),
            Axis::H => assert_eq!(h, size),
            Axis::U => {
                assert_eq!((b, h), (1, size));
                assert_eq!(u, size + 1);
            }
            Axis::A => assert_eq!(a, size - 1),
            Axis::P => assert_eq!(p, size - 1),
            Axis::O => assert_eq!(o, size),
        }
        population
    }
    pub fn axes(&self) -> (usize, usize, usize, usize, usize, usize) {
        let inspection = self.court.world.inspection_port();
        let history = inspection.history_snapshot().unwrap();
        let retention = inspection.retention_snapshot().unwrap();
        let recovery = inspection.recovery_snapshot().unwrap();
        (
            1 + self.branches.len(),
            history.installed_commits(),
            retention.unique_pins(),
            recovery.reserved(),
            recovery.installed(),
            retention.observations(),
        )
    }
    pub fn finish(self) {
        let Self {
            court,
            root,
            branches,
            observations,
            attempts,
            partials,
        } = self;
        drop(attempts);
        for partial in partials {
            let handle = partial.recovery_handle();
            drop(partial);
            court
                .world
                .recovery_port()
                .release_effects(&handle, 0)
                .unwrap();
        }
        for head in branches {
            drop(
                court
                    .world
                    .branch_port()
                    .retire_product_branch(&head)
                    .unwrap(),
            );
        }
        drop((observations, root));
        court.finish();
    }
}
