//! A runtime definition successor uses the same original owner journals as advances.
use super::*;
use worth_foundational::facade::*;
use worth_runtime_bridge::facade::*;

pub(super) fn successor(
    predecessor: &BridgeInstalledConditionalLowering,
) -> BridgeOwnedConditionalInstallationRequest {
    BridgeOwnedConditionalInstallationRequest {
        contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
            identity: Arc::from(predecessor.contract().identity()),
            dependency_count: 0,
            condition_dependency_ordinals: Vec::new(),
            condition: BridgeConditionalCondition::Always,
            dependency_comparator: SignalConditionalVersionComparator::Exact,
            output_comparator: SignalConditionalVersionComparator::Exact,
            artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
        }),
        location: predecessor.location().clone(),
        dependencies: Vec::new(),
        providers: program::providers(&BridgeConditionalCondition::Always),
    }
}
#[test]
fn conditional_definition_publication_resumes_its_original_and_reopens_only_the_acting_branch() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit(medium.clone(), &execution);
    let branch = reference.definitions[0].branch();
    let port = owner.signal_owner().source_port();
    let descriptor = port.observe(branch).unwrap();
    let basis = port.readmit_descriptor(&descriptor).unwrap();
    let original = owner.with_authored_branch(branch, |view| {
        let original = view.conditional_definitions().next().unwrap();
        assert!(matches!(owner.begin_conditional_definition_publication(&execution, &basis, &original, successor(&original)),
            Err(BridgeDurableDefinitionPublicationDenial::Branch(BridgeAuthoredBranchDenial::ProjectionContactHeld { branch: held })) if held == branch));
        original
    }).unwrap();
    let task = owner
        .begin_conditional_definition_publication(
            &execution,
            &basis,
            &original,
            successor(&original),
        )
        .unwrap();
    let original_task = task.original();
    assert_eq!(
        port.observe(branch).unwrap(),
        descriptor,
        "preparation moves no canonical head"
    );
    assert!(
        matches!(owner.with_authored_branch(branch, |_| ()), Err(BridgeAuthoredBranchDenial::Pending { branch: held, original: retained }) if held == branch && retained == original_task)
    );
    drop(task);
    let resumed = owner
        .signal_owner()
        .mutation_port()
        .resume_original(original_task)
        .unwrap();
    let output = owner_fixture::finish(resumed, &execution);
    assert_ne!(
        output.artifact().descriptor().definition(),
        descriptor.definition()
    );
    let new_reference = output.artifact().descriptor().definition();
    let actual = owner
        .with_authored_branch(branch, |view| {
            view.conditional_definitions().next().unwrap()
        })
        .unwrap();
    assert!(matches!(
        actual.contract().condition(),
        BridgeConditionalCondition::Always
    ));
    assert!(
        matches!(owner.begin_conditional_definition_publication(&execution, output.branch(), &original, successor(&original)),
        Err(BridgeDurableDefinitionPublicationDenial::NotCurrent { branch: held, node }) if held == branch && node == original.signal_node())
    );
    let source = owner
        .signal_owner()
        .read_definition_source(output.artifact().descriptor(), &execution)
        .unwrap();
    let old_occurrence = reference.definitions[0]
        .conditional_occurrences()
        .next()
        .unwrap();
    let new_occurrence = source.conditional_occurrences().next().unwrap();
    assert_eq!(new_occurrence.node(), old_occurrence.node());
    assert!(
        new_occurrence.occurrence().ordinal()
            > reference
                .definitions
                .iter()
                .flat_map(|d| d.conditional_occurrences())
                .map(|r| r.occurrence().ordinal())
                .max()
                .unwrap()
    );
    assert_eq!(source.predecessor(), Some(descriptor.definition()));
    let bytes = source.as_bytes().to_vec();
    let definition_count = medium
        .capture_recovery_source(&execution)
        .unwrap()
        .frontier(SignalDurableJournal::Definitions, &execution)
        .unwrap();
    assert_eq!(definition_count, 4);
    drop((actual, original, output, basis, port, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        2,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let owner = readmit(medium.clone(), &execution);
    for (identity, _) in owner.signal_owner().source_port().branches().unwrap() {
        let descriptor = owner
            .signal_owner()
            .source_port()
            .observe(identity)
            .unwrap();
        let definitions: Vec<_> = owner
            .with_authored_branch(identity, |view| view.conditional_definitions().collect())
            .unwrap();
        if identity == branch {
            assert_eq!(descriptor.definition(), new_reference);
            assert!(matches!(
                definitions[0].contract().condition(),
                BridgeConditionalCondition::Always
            ));
            let recorded = owner
                .signal_owner()
                .read_definition_source(&descriptor, &execution)
                .unwrap();
            assert_eq!(recorded.as_bytes(), bytes);
            let handle = owner
                .signal_owner()
                .source_port()
                .readmit_descriptor(&descriptor)
                .unwrap();
            let (image, runtime) = owner
                .signal_owner()
                .source_port()
                .capture_authored_images(&handle, &execution)
                .unwrap();
            assert_eq!(image.as_bytes(), recorded.image_bytes());
            assert_eq!(runtime.as_bytes(), recorded.runtime_bytes());
        } else {
            assert!(matches!(
                definitions[0].contract().condition(),
                BridgeConditionalCondition::OnDemand
            ));
            assert_eq!(
                descriptor.definition(),
                reference.definitions[(identity.ordinal() - 1) as usize].reference()
            );
        }
        assert!(matches!(
            definitions[1].contract().condition(),
            BridgeConditionalCondition::OnDemand
        ));
        assert_eq!(
            owner
                .with_authored_branch(identity, |view| view
                    .conditional_retention()
                    .retained_definition_candidates())
                .unwrap(),
            0
        );
    }
    assert_eq!(
        medium
            .capture_recovery_source(&execution)
            .unwrap()
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        definition_count,
        "readmission must not republish the conditional effect"
    );
    let calls = AtomicUsize::new(0);
    demand(
        &owner,
        reference.program.value,
        &execution,
        &calls,
        derived_reference::value(&execution),
    );
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    drop(owner);
    media::close(serving, medium, &execution);
}
