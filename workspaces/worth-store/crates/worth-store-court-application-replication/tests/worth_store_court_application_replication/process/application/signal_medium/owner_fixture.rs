//! Original journals are authored from the same public memory owner used as the oracle.
use super::{drive, medium_fixture};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_signal::facade::AspectMask;
use worth_signal::facade::*;
pub(super) type Runtime = SignalRuntime<(), (), (), (), ()>;
pub(super) type Owner = SignalDurableOwner<(), (), (), (), ()>;

pub(super) fn vocabulary() -> SignalDurableTierVocabulary<()> {
    SignalDurableTierVocabulary::declare([((), "base")]).unwrap()
}
pub(super) fn program() -> (Box<Runtime>, NodeId, NodeId) {
    let mut graph = Box::new(SignalGraph::new());
    let lowering = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&lowering).unwrap();
    let value = graph.node().on_demand().build();
    let conditional = graph.create_node();
    let capability = graph.admit_installed_node(conditional).unwrap();
    graph
        .install_conditional_contract(
            &lowering,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::DeltaThreshold(
                    SignalDeltaThresholdContract::new(
                        AspectValue::UInt64(37),
                        "authored-units",
                        SignalThresholdValueFamily::Integer,
                        SignalThresholdComparisonDomain::AbsoluteDifference,
                        SignalThresholdBoundary::Inclusive,
                    ),
                ),
                dependency_aspects: AspectMask::ALL,
                trigger_aspects: AspectMask::ALL,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    let mut runtime: Box<Runtime> = Box::new(
        SignalRuntime::builder_from_boxed_graph(graph)
            .with_kernel_defaults()
            .build(),
    );
    recipe(&mut runtime);
    (runtime, value, conditional)
}
pub(super) fn recipe(
    runtime: &mut Runtime,
) -> RecipeInstance<(), fn(&mut EvaluationContext<'_, ()>) -> Result<AspectVersion, SignalError>> {
    fn evaluate(_: &mut EvaluationContext<'_, ()>) -> Result<AspectVersion, SignalError> {
        Ok(AspectVersion::from_updates([(Aspect::new(0), 83)]))
    }
    runtime
        .define(worth_signal::facade::runtime::Recipe::new(
            "authored-keyed-source",
            (),
            evaluate as fn(&mut EvaluationContext<'_, ()>) -> Result<AspectVersion, SignalError>,
        ))
        .unwrap()
}
pub(super) fn readmit(
    medium: Arc<dyn SignalDurableMedium>,
    execution: &ExecutionRequest<'_, '_>,
) -> Owner {
    program()
        .0
        .readmit_authored_owner(
            medium,
            SignalRecordedIdentityContribution::empty(medium_fixture::owner()),
            vocabulary(),
            &mut worth_signal::facade::durability::SignalOnlyOwnerInstallation,
            execution,
        )
        .unwrap()
}
pub(super) struct Reference {
    pub(super) memory: Box<Runtime>,
    pub(super) node: NodeId,
    pub(super) heads: BTreeMap<DurableSignalBranchIdentity, SignalDurableBranchDescriptor>,
    pub(super) images: BTreeMap<DurableSignalBranchIdentity, (Vec<u8>, Vec<u8>)>,
}
fn transition(
    memory: &Runtime,
    branch: DurableSignalBranchIdentity,
    ordinal: u64,
    predecessor: Option<SignalDefinitionTransitionReference>,
    conditional: Option<NodeId>,
    execution: &ExecutionRequest<'_, '_>,
) -> SignalDefinitionTransitionArtifact {
    let image = memory
        .graph()
        .capture_authored_image(execution, branch)
        .unwrap();
    let runtime = memory
        .config()
        .capture_authored_image(memory.graph(), &vocabulary(), branch, execution)
        .unwrap();
    let occurrences = conditional
        .into_iter()
        .map(|node| {
            DurableSignalConditionalDefinitionOccurrence::from_recorded(
                DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation()),
                DurableSignalConditionalOccurrence::from_recorded(branch.owner(), ordinal),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let bytes = ExecutionRequestArrayBuilder::allocate(*execution, 0)
        .unwrap()
        .seal()
        .unwrap()
        .into_immutable_bytes();
    SignalDefinitionTransitionArtifact::capture(
        execution,
        SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), ordinal),
        branch,
        predecessor,
        &image,
        &runtime,
        &occurrences,
        &SignalBridgeAuthoredSection::from_bridge_bytes(bytes).unwrap(),
    )
    .unwrap()
}
fn descriptor(
    branch: DurableSignalBranchIdentity,
    generation: u64,
    admission: u64,
    definition: &SignalDefinitionTransitionArtifact,
) -> SignalDurableBranchDescriptor {
    SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, branch.ordinal() + 1),
            generation,
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(branch.owner(), admission),
        definition.reference(),
    )
    .unwrap()
}
fn publish(
    medium: Option<&dyn SignalDurableMedium>,
    definition: &SignalDefinitionTransitionArtifact,
    advance: &SignalBranchAdvanceArtifact,
    execution: &ExecutionRequest<'_, '_>,
) {
    let Some(medium) = medium else { return };
    assert!(matches!(
        drive(|| medium.advance_definition(definition, execution)).fate(),
        DurablePublicationFate::Committed
    ));
    assert!(matches!(
        drive(|| medium.advance_branch(advance, execution)).fate(),
        DurablePublicationFate::Committed
    ));
}
pub(super) fn seed(
    medium: &dyn SignalDurableMedium,
    execution: &ExecutionRequest<'_, '_>,
) -> Reference {
    seed_inner(Some(medium), execution)
}
pub(super) fn memory_only(execution: &ExecutionRequest<'_, '_>) -> Reference {
    seed_inner(None, execution)
}
fn seed_inner(
    medium: Option<&dyn SignalDurableMedium>,
    execution: &ExecutionRequest<'_, '_>,
) -> Reference {
    let (mut memory, node, conditional) = program();
    let main = memory.current_branch();
    let branch = DurableSignalBranchIdentity::from_recorded(medium_fixture::owner(), main.id.0);
    let mut base_graph = SignalGraph::new();
    base_graph.node().on_demand().build();
    let base = SignalRuntime::build_for::<()>(base_graph);
    let initial = transition(&base, branch, 1, None, None, execution);
    let initial_head = descriptor(branch, 0, 1, &initial);
    publish(
        medium,
        &initial,
        &SignalBranchAdvanceArtifact::capture(
            execution,
            SignalBranchAdvanceIdentity::from_recorded(branch.owner(), 1),
            initial_head,
            None,
            SignalAuthoredBranchChange::Created {
                name: main.name.clone(),
                parent: None,
            },
        )
        .unwrap(),
        execution,
    );
    let prior = memory.observe_signal_branch_basis(main.clone()).unwrap();
    let moved = memory
        .advance_signal_branch(*execution, &mut (), &prior, |_| Ok(()))
        .unwrap();
    let full = transition(
        &memory,
        branch,
        2,
        Some(initial.reference()),
        Some(conditional),
        execution,
    );
    let mut parent = descriptor(
        branch,
        moved.advanced_basis().observation().generation().get(),
        2,
        &full,
    );
    publish(
        medium,
        &full,
        &SignalBranchAdvanceArtifact::capture(
            execution,
            SignalBranchAdvanceIdentity::from_recorded(branch.owner(), 2),
            parent.clone(),
            None,
            SignalAuthoredBranchChange::Advanced,
        )
        .unwrap(),
        execution,
    );
    let mut heads = BTreeMap::from([(branch, parent.clone())]);
    let mut basis = moved.into_basis();
    for (ordinal, name) in [(3, "child"), (4, "grandchild")] {
        let created = memory.fork_signal_branch(name, &basis).unwrap();
        let handle = created.created_branch().clone();
        basis = created.created_basis().clone();
        memory.switch_branch(handle.clone()).unwrap();
        let branch =
            DurableSignalBranchIdentity::from_recorded(medium_fixture::owner(), handle.id.0);
        let definition = transition(
            &memory,
            branch,
            ordinal,
            Some(parent.definition()),
            Some(conditional),
            execution,
        );
        let head = descriptor(
            branch,
            basis.observation().generation().get(),
            ordinal,
            &definition,
        );
        publish(
            medium,
            &definition,
            &SignalBranchAdvanceArtifact::capture(
                execution,
                SignalBranchAdvanceIdentity::from_recorded(branch.owner(), ordinal),
                head.clone(),
                None,
                SignalAuthoredBranchChange::Created {
                    name: handle.name,
                    parent: Some(parent),
                },
            )
            .unwrap(),
            execution,
        );
        heads.insert(branch, head.clone());
        parent = head;
    }
    let mut images = BTreeMap::new();
    for handle in memory.known_branches() {
        memory.switch_branch(handle.clone()).unwrap();
        let branch =
            DurableSignalBranchIdentity::from_recorded(medium_fixture::owner(), handle.id.0);
        let image = memory
            .graph()
            .capture_authored_image(execution, branch)
            .unwrap();
        let runtime = memory
            .config()
            .capture_authored_image(memory.graph(), &vocabulary(), branch, execution)
            .unwrap();
        images.insert(
            branch,
            (image.as_bytes().to_vec(), runtime.as_bytes().to_vec()),
        );
    }
    Reference {
        memory,
        node,
        heads,
        images,
    }
}
pub(super) fn finish(
    mut task: SignalDurableAdvanceTask,
    execution: &ExecutionRequest<'_, '_>,
) -> SignalDurableAdvanceOutput {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match task
            .drive(execution)
            .unwrap_or_else(|refusal| panic!("{:?}", refusal.denial()))
        {
            SignalDurableAdvanceProgress::Pending(next) => {
                assert!(std::time::Instant::now() < deadline);
                task = next;
                std::thread::yield_now();
            }
            SignalDurableAdvanceProgress::Finished(completion) => {
                return completion
                    .into_committed()
                    .unwrap_or_else(|completion| panic!("{:?}", completion.fate()))
            }
        }
    }
}
