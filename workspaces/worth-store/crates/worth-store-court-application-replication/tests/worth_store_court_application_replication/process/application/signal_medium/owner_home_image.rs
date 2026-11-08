//! Owner image courts use actual Store originals, without claiming Home selection.
use super::{home_image_capture_probe as probe, medium_fixture as media, owner_fixture as fixture};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::*;
use worth_signal::facade::{durability::*, *};
use worth_store_application::StoreSignalMedium;

pub(super) fn reopen(
    root: &std::path::Path,
    image: SignalAuthoredHomeImage,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    Arc<worth_store::physical_runtime::ServingPhysicalRuntime>,
    Arc<StoreSignalMedium>,
) {
    let (serving, placement) = super::physical_open::open(root, false);
    let binding = SignalDurableHomeBinding::from_admitted_adapter(
        media::owner(),
        HomeIncarnation::from_recorded(2).unwrap(),
        [8; 32],
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap(),
    );
    StoreSignalMedium::from_serving(serving, binding, Some(image), placement, execution).unwrap()
}
#[derive(Debug)]
struct PrefixFence {
    source: Box<dyn SignalDurableReadSource>,
    definitions: u64,
    advances: u64,
    reads: Arc<AtomicUsize>,
}
impl SignalDurableReadSource for PrefixFence {
    fn owner(&self) -> DurableSignalOwnerIdentity {
        self.source.owner()
    }
    fn identity_contribution(&self) -> SignalRecordedIdentityContribution {
        self.source.identity_contribution()
    }
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalDurableMediumDenial> {
        self.source.frontier(journal, execution)
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, SignalDurableMediumDenial> {
        let prefix = match journal {
            SignalDurableJournal::Definitions => self.definitions,
            SignalDurableJournal::Advances => self.advances,
        };
        assert!(
            position > prefix,
            "image prefix must not be reread from its original tail"
        );
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.source.read(journal, position, execution)
    }
}
#[test]
fn owner_image_prefix_and_original_tail_rebuild_several_branches_cold_under_a_new_build() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut reference = fixture::seed(medium.as_ref(), &execution);
    let observed = probe::ObservedMedium::new(medium.clone());
    let owner = fixture::readmit(observed.clone(), &execution);
    let (&branch, descriptor) = reference.heads.first_key_value().unwrap();
    let head = owner.source_port().readmit_descriptor(descriptor).unwrap();
    let original_point = head.source_point().clone();
    *observed.observation.publication.lock().unwrap() = Some(probe::PublicationProbe {
        port: owner.mutation_port(),
        source: owner.source_port(),
        head: descriptor.clone(),
    });
    let image = owner.capture_authored_home_image(&execution).unwrap();
    assert!(observed.observation.refused.load(Ordering::Relaxed) > 0);
    assert_eq!(observed.observation.effects.load(Ordering::Relaxed), 0);
    observed.observation.publication.lock().unwrap().take();
    let before = image.covered_through(SignalDurableJournal::Advances);
    let memory_handle = reference
        .memory
        .branch_handle(SignalBranchId(branch.ordinal()))
        .unwrap();
    let memory_basis = reference
        .memory
        .observe_signal_branch_basis(memory_handle)
        .unwrap();
    let mut expected = None;
    reference
        .memory
        .advance_signal_branch(execution, &mut (), &memory_basis, |tx| {
            expected = Some(tx.target(reference.node).read(&|_: &mut EvaluationContext<
                '_,
                (),
            >| {
                Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
            })?);
            Ok(())
        })
        .unwrap();
    let calls = AtomicUsize::new(0);
    let evaluate = |_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
    };
    let output = fixture::finish(
        owner
            .mutation_port()
            .begin_advance(execution, &head, &mut (), |tx| {
                assert_eq!(Some(tx.target(reference.node).read(&evaluate)?), expected);
                Ok(())
            })
            .unwrap(),
        &execution,
    );
    assert_eq!(
        output.artifact().identity().ordinal(),
        before + 1,
        "capture refusals issue no original identities"
    );
    let tail_point = output.evidence().source_point();
    let reads = Arc::new(AtomicUsize::new(0));
    let fenced = image
        .with_original_tail(
            Box::new(PrefixFence {
                source: medium.capture_recovery_source(&execution).unwrap(),
                definitions: image.covered_through(SignalDurableJournal::Definitions),
                advances: before,
                reads: Arc::clone(&reads),
            }),
            &execution,
        )
        .unwrap();
    for journal in [
        SignalDurableJournal::Definitions,
        SignalDurableJournal::Advances,
    ] {
        let expected = medium
            .capture_recovery_source(&execution)
            .unwrap()
            .read(journal, 1, &execution)
            .unwrap();
        assert_eq!(
            fenced.read(journal, 1, &execution).unwrap().bytes(),
            expected.bytes()
        );
    }
    assert_eq!(reads.load(Ordering::Relaxed), 0);
    assert_eq!(
        fenced
            .read(
                SignalDurableJournal::Advances,
                tail_point.position(),
                &execution
            )
            .unwrap()
            .bytes(),
        output.artifact().as_bytes()
    );
    assert_eq!(reads.load(Ordering::Relaxed), 1);
    drop((fenced, output, head, owner, observed));
    media::close(serving, medium, &execution);
    let (serving, medium) = reopen(directory.path(), image, &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    assert_eq!(
        owner
            .source_port()
            .branches()
            .unwrap()
            .iter()
            .map(|(_, h)| h.clone())
            .collect::<Vec<_>>(),
        reference.memory.known_branches()
    );
    for (&branch, (graph, runtime)) in &reference.images {
        let current = owner.source_port().observe(branch).unwrap();
        let handle = owner.source_port().readmit_descriptor(&current).unwrap();
        let actual = owner
            .source_port()
            .capture_authored_images(&handle, &execution)
            .unwrap();
        assert_eq!(actual.0.as_bytes(), graph);
        assert_eq!(actual.1.as_bytes(), runtime);
    }
    let historical = owner
        .readmit_source_point(&original_point, &execution)
        .unwrap();
    assert_eq!(historical.source_point(), &original_point);
    let actual = owner
        .source_port()
        .readmit_descriptor(tail_point.descriptor())
        .unwrap();
    assert_eq!(actual.source_point(), &tail_point);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let task = owner
        .mutation_port()
        .begin_advance(execution, &actual, &mut (), |tx| {
            assert_eq!(Some(tx.target(reference.node).read(&evaluate)?), expected);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        calls.load(Ordering::Relaxed),
        2,
        "changed-build derived state recomputes on first demand"
    );
    drop(fixture::finish(task, &execution));
    drop((historical, actual, owner));
    media::close(serving, medium, &execution);
}

#[test]
fn image_capture_names_every_original_and_a_failed_capture_releases_its_publication_gate() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let observed = probe::ObservedMedium::new(medium.clone());
    let owner = fixture::readmit(observed.clone(), &execution);
    let effects = AtomicUsize::new(0);
    let mut tasks = Vec::new();
    for descriptor in reference.heads.values().take(2) {
        let head = owner.source_port().readmit_descriptor(descriptor).unwrap();
        tasks.push(
            owner
                .mutation_port()
                .begin_advance(execution, &head, &mut (), |_| {
                    effects.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
                .unwrap(),
        );
    }
    let expected = tasks.iter().map(|task| task.original()).collect::<Vec<_>>();
    let originals = match owner.capture_authored_home_image(&execution) {
        Err(SignalDurableAdvanceDenial::CheckpointOriginalsPending { originals }) => originals,
        other => panic!("capture must name both actual originals: {other:?}"),
    };
    assert_eq!(originals.elements(), expected);
    drop(tasks);
    for original in expected {
        drop(fixture::finish(
            owner.mutation_port().resume_original(original).unwrap(),
            &execution,
        ));
    }
    assert_eq!(effects.load(Ordering::Relaxed), 2);
    observed
        .observation
        .fail_read_once
        .store(true, Ordering::Relaxed);
    assert!(matches!(
        owner.capture_authored_home_image(&execution),
        Err(SignalDurableAdvanceDenial::Readmission(
            SignalOwnerReadmissionDenial::Medium(SignalDurableMediumDenial::Unavailable)
        ))
    ));
    let descriptor = owner
        .source_port()
        .observe(*reference.heads.keys().next().unwrap())
        .unwrap();
    let head = owner.source_port().readmit_descriptor(&descriptor).unwrap();
    drop(fixture::finish(
        owner
            .mutation_port()
            .begin_advance(execution, &head, &mut (), |_| {
                effects.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .unwrap(),
        &execution,
    ));
    assert_eq!(effects.load(Ordering::Relaxed), 3);
    assert!(owner.capture_authored_home_image(&execution).is_ok());
    drop((head, owner, observed));
    media::close(serving, medium, &execution);
}

#[test]
fn owner_image_refuses_foreign_binding_short_tail_unknown_version_and_checksum_damage() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let image = owner.capture_authored_home_image(&execution).unwrap();
    let mut bytes = image.as_bytes().to_vec();
    bytes[8..12].copy_from_slice(&999u32.to_le_bytes());
    let bytes = ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice()));
    assert!(matches!(
        SignalAuthoredHomeImage::readmit_region(&execution, bytes.clone(), 0..bytes.len()),
        Err(SignalOwnerReadmissionDenial::Frame(
            SignalDurableFrameDenial::UnsupportedVersion(_)
        ))
    ));
    let mut bytes = image.as_bytes().to_vec();
    *bytes.last_mut().unwrap() ^= 1;
    let bytes = ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice()));
    assert!(matches!(
        SignalAuthoredHomeImage::readmit_region(&execution, bytes.clone(), 0..bytes.len()),
        Err(SignalOwnerReadmissionDenial::Frame(
            SignalDurableFrameDenial::ChecksumMismatch
        ))
    ));
    drop(owner);
    media::close(serving, medium, &execution);
    let empty = tempfile::tempdir().unwrap();
    let (serving, placement) = super::physical_open::open(empty.path(), true);
    let foreign = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([9; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    );
    let binding = SignalDurableHomeBinding::from_admitted_adapter(
        foreign,
        HomeIncarnation::from_recorded(2).unwrap(),
        [8; 32],
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap(),
    );
    let refused =
        match StoreSignalMedium::from_serving(serving, binding, Some(image), placement, &execution)
        {
            Err(refused) => refused,
            Ok(_) => panic!("foreign image binding must retain the original input"),
        };
    assert!(matches!(
        refused.cause(),
        SignalDurableMediumDenial::ForeignOwner
    ));
    let (serving, _, image, placement) = refused.into_parts();
    let binding = SignalDurableHomeBinding::from_admitted_adapter(
        media::owner(),
        HomeIncarnation::from_recorded(2).unwrap(),
        [8; 32],
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap(),
    );
    let (serving, medium) =
        StoreSignalMedium::from_serving(serving, binding, image, placement, &execution).unwrap();
    assert!(
        matches!(medium.capture_recovery_source(&execution), Err(SignalDurableMediumDenial::CheckpointSourceBehind { journal: SignalDurableJournal::Definitions, checkpoint, observed:0 }) if checkpoint > 0)
    );
    media::close(serving, medium, &execution);
}
