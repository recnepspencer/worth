//! Real World publication demands the actual Home's authored Signal owner.
use super::{WorldHomeFixture, WorldHomeFixtureRefusal};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_signal::facade::*;
fn program() -> (Box<SignalRuntime<(), (), (), (), ()>>, NodeId) {
    let mut graph = SignalGraph::new();
    let value = graph.node().on_demand().build();
    (Box::new(SignalRuntime::build_for::<()>(graph)), value)
}
pub fn open_with_derived_signal(
    request: worth_store_application::ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldHomeFixture, WorldHomeFixtureRefusal> {
    let (signal, _) = program();
    super::open_definition(
        request,
        execution,
        super::definition::authored_with_signal(64, signal),
    )
}
pub fn reference_signal_value(execution: &ExecutionRequest<'_, '_>) -> u64 {
    let (mut memory, node) = program();
    let basis = memory
        .observe_signal_branch_basis(memory.current_branch())
        .unwrap();
    let mut value = None;
    memory
        .advance_signal_branch(*execution, &mut (), &basis, |tx| {
            value = Some(tx.target(node).read(
                &|_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
                    Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
                },
            )?);
            Ok(())
        })
        .unwrap();
    value.unwrap().get(Aspect::new(0))
}
impl WorldHomeFixture {
    /// Compare both authored codecs against a fresh ordinary memory programme.
    pub fn assert_derived_signal_authored_reference(&self, execution: &ExecutionRequest<'_, '_>) {
        let source = self.home.standalone_signal_source().unwrap();
        let branches = source.branches().unwrap();
        assert_eq!(branches.len(), 3);
        for (branch, _) in branches {
            let descriptor = source.observe(branch).unwrap();
            let basis = source.readmit_descriptor(&descriptor).unwrap();
            let (actual_graph, actual_runtime) =
                source.capture_authored_images(&basis, execution).unwrap();
            let (memory, _) = program();
            let graph = memory
                .graph()
                .capture_authored_image(execution, branch)
                .unwrap();
            let vocabulary =
                worth_signal::facade::durability::SignalDurableTierVocabulary::declare([(
                    (),
                    "base",
                )])
                .unwrap();
            let runtime = memory
                .config()
                .capture_authored_image(memory.graph(), &vocabulary, branch, execution)
                .unwrap();
            assert_eq!(actual_graph.as_bytes(), graph.as_bytes());
            assert_eq!(actual_runtime.as_bytes(), runtime.as_bytes());
        }
    }
    pub fn assert_signal_descriptor_refusals(&self, execution: &ExecutionRequest<'_, '_>) {
        use worth_foundational::facade::*;
        use worth_signal::facade::durability::*;
        let source = self.home.standalone_signal_source().unwrap();
        let branch = source.branches().unwrap()[0].0;
        let original = source.observe(branch).unwrap();
        drop(source.readmit_descriptor(&original).unwrap());
        let foreign = DurableSignalOwnerIdentity::from_recorded(
            branch.owner().home(),
            SignalOwnerSlotIdentity::from_recorded(branch.owner().slot().get() ^ 1),
        );
        let other = DurableSignalBranchIdentity::from_recorded(foreign, branch.ordinal());
        let descriptor = SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(
                DurableSignalCellIncarnation::from_recorded(other, original.cell().ordinal()),
                original.generation().ordinal(),
            ),
            None,
            DurableSignalAdmissionIdentity::from_recorded(foreign, original.admission().ordinal()),
            SignalDefinitionTransitionReference::from_recorded(
                SignalDefinitionTransitionIdentity::from_recorded(
                    foreign,
                    original.definition().identity().ordinal(),
                ),
                original.definition().digest(),
            ),
        )
        .unwrap();
        assert!(matches!(
            source.readmit_descriptor(&descriptor),
            Err(SignalDurableSourceDenial::ForeignOwner)
        ));
        let mut bytes = original.encode_bytes(execution).unwrap().bytes().to_vec();
        bytes[8..12].copy_from_slice(&999_u32.to_le_bytes());
        assert!(matches!(
            source.readmit_descriptor_bytes(execution, &bytes),
            Err(SignalDurableSourceDenial::Frame(
                SignalDurableFrameDenial::UnsupportedVersion(_)
            ))
        ));
    }
    /// Fork two Signal owners from the exact root; Native remains its exact source.
    pub fn fork_derived_signal_branches(&self, execution: &ExecutionRequest<'_, '_>) {
        let source = self
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        let port = self.home.world().branch_port();
        let cancellation = RuntimeWorldCancellationSource::new();
        for ordinal in 0..2 {
            let name = format!("derived-sibling-{ordinal}");
            let intent = ProductBranchCreationIntent::from_source(
                name.clone(),
                ProductBranchCreationPlans::new(
                    RelationalBranchCreationPlan::ReuseExact,
                    SignalBranchCreationPlan::ForkExact {
                        target: worth_signal::facade::branch::validate_signal_branch_name(name)
                            .unwrap(),
                    },
                ),
            )
            .unwrap();
            let mut outcome = port
                .create_home_product_branch(
                    execution,
                    RuntimeWorldHomeBranchCreationRequest::new(
                        source.clone(),
                        self.admit_initial_program_source(&source),
                        intent,
                    ),
                    &cancellation.token(),
                )
                .unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            loop {
                match outcome {
                    RuntimeWorldBranchCreationOutcome::Performed(observation) => {
                        drop(observation);
                        break;
                    }
                    RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => {
                        let handle = effects.recovery_handle();
                        drop(effects);
                        assert!(std::time::Instant::now() < deadline);
                        execution.consult().unwrap();
                        std::thread::yield_now();
                        outcome = port
                            .resume_creation(execution, &handle, &cancellation.token())
                            .unwrap();
                    }
                    RuntimeWorldBranchCreationOutcome::NoEffect(cause) => {
                        panic!("actual Signal fork refused: {cause:?}")
                    }
                    RuntimeWorldBranchCreationOutcome::ReleasedOriginal(cause) => {
                        panic!("actual Signal fork released: {cause:?}")
                    }
                }
            }
        }
    }
    /// Demand every actual branch; independent cold caches must each execute once.
    pub fn demand_derived_signal(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        calls: &AtomicUsize,
    ) -> u64 {
        let branches = self
            .home
            .world()
            .observation_port()
            .observe_installed_product_branches(execution)
            .unwrap();
        assert_eq!(branches.branches().len(), 3);
        let mut value = None;
        for expected in branches.branches() {
            let actual = self.demand_signal_branch(execution, calls, expected.clone());
            if let Some(previous) = value {
                assert_eq!(previous, actual);
            } else {
                value = Some(actual);
            }
        }
        value.unwrap()
    }
    fn demand_signal_branch(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        calls: &AtomicUsize,
        expected: ProductBranchObservation,
    ) -> u64 {
        let (_, node) = program();
        let port = self.home.world().publication_port();
        let cancellation = RuntimeWorldCancellationSource::new();
        let pending = port
            .prepare_query_publication_with_signal(
                execution,
                expected.clone(),
                self.admit_initial_program_source(&expected),
                None,
                &cancellation.token(),
                None,
            )
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut opening = pending.advance(execution);
        let open = loop {
            match opening {
                WorldOpenAttemptOutcome::Opened(open) => break open,
                WorldOpenAttemptOutcome::Pending(pending)
                | WorldOpenAttemptOutcome::Unresolved(pending)
                | WorldOpenAttemptOutcome::EvidenceRequired(pending) => {
                    execution.consult().unwrap();
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::yield_now();
                    opening = pending.advance(execution);
                }
                WorldOpenAttemptOutcome::Refused(cause) => panic!("actual Signal open: {cause:?}"),
            }
        };
        let prepared = open
            .prepare_signal(self.home.relational().runtime(), execution, None)
            .unwrap();
        let mut value = None;
        let mut outcome = port.execute_durable_with_signal(
            *execution,
            prepared,
            &mut (),
            &cancellation.token(),
            |tx| {
                value = Some(tx.target(node).read(
                    &|_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
                    },
                )?);
                Ok(())
            },
        );
        loop {
            if let WorldDurablePublicationOutcome::Pending(pending) = outcome {
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
                outcome = pending.advance(execution);
                continue;
            }
            let WorldDurablePublicationOutcome::Performed(performed) = outcome else {
                panic!("actual Signal demand refused: {outcome:?}");
            };
            drop(performed.consume());
            break;
        }
        value.unwrap().get(Aspect::new(0))
    }
}
