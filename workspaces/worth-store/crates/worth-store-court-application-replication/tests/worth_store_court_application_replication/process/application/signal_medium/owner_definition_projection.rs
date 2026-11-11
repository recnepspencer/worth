//! A post-commit producer unwind cannot lose or repeat the original owner effect.
use super::{medium_fixture as media, owner_fixture as fixture};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_signal::facade::*;

struct PanickingProjection;
impl SignalDurableDefinitionPublication for PanickingProjection {
    fn prepare(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalDefinitionPublicationContext<'_>,
    ) -> Result<SignalBridgeAuthoredSection, SignalDurableAdvanceDenial> {
        SignalOnlyDefinitionPublication.prepare(execution, context)
    }
}
fn program() -> (Box<fixture::Runtime>, SignalAspectLoweringOwner) {
    let mut graph = Box::new(SignalGraph::new());
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let mut runtime = Box::new(
        SignalRuntime::builder_from_boxed_graph(graph)
            .with_kernel_defaults()
            .build(),
    );
    fixture::recipe(&mut runtime);
    (runtime, claimant)
}
#[test]
fn projection_unwind_retains_the_committed_original_for_exact_delivery_without_republication() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let _reference = fixture::seed(medium.as_ref(), &execution);
    let (program, claimant) = program();
    let mut owner = program
        .readmit_authored_owner(
            medium.clone(),
            SignalRecordedIdentityContribution::empty(media::owner()),
            fixture::vocabulary(),
            &mut SignalOnlyOwnerInstallation,
            &execution,
        )
        .unwrap();
    let contracts = owner.take_installation_contracts();
    let (&branch, rows) = contracts.first_key_value().unwrap();
    let port = owner.source_port();
    let before = port.observe(branch).unwrap();
    let basis = port.readmit_descriptor(&before).unwrap();
    let count = medium
        .capture_recovery_source(&execution)
        .unwrap()
        .frontier(SignalDurableJournal::Definitions, &execution)
        .unwrap();
    let task = owner
        .mutation_port()
        .begin_definition_publication(
            execution,
            &basis,
            &mut (),
            claimant,
            rows[0].installed().clone(),
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::Always,
                dependency_aspects: worth_signal::facade::AspectMask::ALL,
                trigger_aspects: worth_signal::facade::AspectMask::ALL,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
            Box::new(PanickingProjection),
        )
        .unwrap();
    let original = task.original();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fixture::finish(
            task, &execution
        )))
        .is_err()
    );
    assert_eq!(
        port.observe(branch).unwrap().generation().ordinal(),
        before.generation().ordinal() + 1
    );
    let delivered = fixture::finish(
        owner.mutation_port().resume_original(original).unwrap(),
        &execution,
    );
    assert_eq!(
        delivered.projection_outcome(),
        SignalDefinitionProjectionOutcome::UnwoundAfterCommit
    );
    assert_eq!(
        owner
            .read_original_advance(original, &execution)
            .unwrap()
            .as_bytes(),
        delivered.artifact().as_bytes()
    );
    assert_eq!(
        medium
            .capture_recovery_source(&execution)
            .unwrap()
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        count + 1
    );
    assert!(
        matches!(owner.mutation_port().resume_original(original), Err(SignalDurableAdvanceDenial::UnknownTask { original: finished }) if finished == original)
    );
    drop((contracts, basis, delivered, port, owner));
    media::close(serving, medium, &execution);
}

impl SignalDurableProjectionPublication for PanickingProjection {
    fn committed(self: Box<Self>, _: &SignalDurableBranchHandle) {
        panic!("projection completion unwinds after the real commit");
    }
}
