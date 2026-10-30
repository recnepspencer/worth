pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_signal::facade::branch::validate_signal_branch_name;
pub(crate) use {
    crate::cost::support::reference_context::population,
    crate::court::ordinary_context::relational,
    crate::court::ordinary_context::CompositeSupplyChainCourt, worth_runtime_world::facade::*,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Axis {
    B,
    H,
    U,
    A,
    P,
    O,
}
impl Axis {
    pub(crate) const ALL: [Self; 6] = [Self::B, Self::H, Self::U, Self::A, Self::P, Self::O];
}
pub(crate) struct Population {
    pub(crate) court: CompositeSupplyChainCourt,
    pub(crate) root: ProductBranchObservation,
    pub(crate) branches: Vec<ProductBranchObservation>,
    pub(crate) observations: Vec<ProductBranchObservation>,
    pub(crate) attempts: Vec<PreparedCompositePublicationWithoutSignal>,
    pub(crate) partials: Vec<ProductUnpublishedOwnerEffects>,
}
pub(crate) fn child(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            request_execution,
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
    pub(crate) fn build(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        axis: Axis,
        size: usize,
    ) -> Self {
        let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
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
                crate::cost::support::reference_context::population::Axis::B => {
                    population.branches.push(child(
                        request_execution,
                        &population.court,
                        &population.root,
                        &name,
                        false,
                        false,
                    ))
                }
                crate::cost::support::reference_context::population::Axis::H => {
                    let receipt = population
                        .court
                        .publish_cargo(&population.root, &(index % 11 + 1).to_string());
                    population.root = population.court.observe(&population.root);
                    drop(receipt);
                }
                crate::cost::support::reference_context::population::Axis::U => {
                    let head = child(
                        request_execution,
                        &population.court,
                        &population.root,
                        &name,
                        true,
                        false,
                    );
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
                crate::cost::support::reference_context::population::Axis::A => {
                    population.attempts.push(population.court.prepare_cargo(
                        request_execution,
                        &population.root,
                        "5",
                    ))
                }
                crate::cost::support::reference_context::population::Axis::P => {
                    let head = child(
                        request_execution,
                        &population.court,
                        &population.root,
                        &name,
                        true,
                        false,
                    );
                    population.partials.push(
                        population
                            .court
                            .partial_after_relational(request_execution, &head),
                    );
                    population.branches.push(head);
                }
                crate::cost::support::reference_context::population::Axis::O => population
                    .observations
                    .push(population.court.observe(&population.root)),
            }
        }
        if matches!(
            axis,
            crate::cost::support::reference_context::population::Axis::B
                | crate::cost::support::reference_context::population::Axis::H
                | crate::cost::support::reference_context::population::Axis::U
                | crate::cost::support::reference_context::population::Axis::O
        ) {
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
            crate::cost::support::reference_context::population::Axis::B => assert_eq!(b, size),
            crate::cost::support::reference_context::population::Axis::H => assert_eq!(h, size),
            crate::cost::support::reference_context::population::Axis::U => {
                assert_eq!((b, h), (1, size));
                assert_eq!(u, size + 1);
            }
            crate::cost::support::reference_context::population::Axis::A => assert_eq!(a, size - 1),
            crate::cost::support::reference_context::population::Axis::P => assert_eq!(p, size - 1),
            crate::cost::support::reference_context::population::Axis::O => assert_eq!(o, size),
        }
        population
    }
    pub(crate) fn axes(&self) -> (usize, usize, usize, usize, usize, usize) {
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
    pub(crate) fn finish(self) {
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
