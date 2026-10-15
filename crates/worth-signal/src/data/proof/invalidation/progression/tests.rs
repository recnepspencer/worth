use crate::data::aspect::{Aspect, AspectMask};
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::proof::invalidation::revalidation::CanonicalDependencyCauseSet;
use crate::data::proof::PartitionScopeSet;

use super::*;

fn node(index: u32) -> NodeId {
    NodeId::new(index, 0)
}

fn source_batch(target: NodeId, revision: DependencyRevision) -> InvalidationWorkBatch {
    let input = CanonicalDependencyCauseSet::from_source_recompute(
        revision,
        5,
        AspectMask::from_aspect(Aspect::new(2)),
        Vec::new(),
    );
    InvalidationWorkBatch::single(InvalidationWorkItem::new(target, revision, input))
}

fn dependency_batch(target: NodeId, revision: DependencyRevision) -> InvalidationWorkBatch {
    let cause = ResolvedDependencyCause::new(
        1,
        target,
        revision,
        node(7),
        Aspect::new(2),
        None,
        10,
        OutputCommitOrdinal(11),
        12,
        PartitionScopeSet::default(),
    );
    let input = CanonicalDependencyCauseSet::from_dependency_causes(vec![cause]);
    InvalidationWorkBatch::single(InvalidationWorkItem::new(target, revision, input))
}

fn source_binding(target: NodeId, revision: DependencyRevision) -> InvalidationOriginBindingAxes {
    InvalidationOriginBindingAxes {
        graph_instance: 1,
        target,
        dependency_revision: revision,
        origin: InvalidationOriginBinding::SourceAdmission { generation: 5 },
    }
}

fn dependency_binding(
    target: NodeId,
    revision: DependencyRevision,
) -> InvalidationOriginBindingAxes {
    InvalidationOriginBindingAxes {
        graph_instance: 1,
        target,
        dependency_revision: revision,
        origin: InvalidationOriginBinding::DependencyCommit {
            cause_set: PendingCauseSetId::EMPTY,
            producer_commit_ordinals: vec![OutputCommitOrdinal(11)],
        },
    }
}

macro_rules! expect_success {
    ($outcome:expr) => {
        match $outcome {
            worth_proof::TransitionOutcome::Success(value) => value,
            other => panic!("expected successful transition, got {other:?}"),
        }
    };
}

#[test]
fn actual_owner_forms_progress_source_work_through_ready_execution() {
    let target = node(3);
    let revision = DependencyRevision(4);
    let admitted = expect_success!(AdmittedSourceRecompute::admit(
        source_batch(target, revision),
        source_binding(target, revision),
    ));
    let resolved = ResolvedInvalidationWork::from_source(admitted);
    let lowered = LoweredInvalidationBatch::lower(
        resolved,
        InvalidationReadinessEpoch(6),
        InvalidationStageOrder { stage: 7, order: 8 },
    );
    let current = lowered.binding().clone();
    let ready = expect_success!(ReadyInvalidationBatch::admit(lowered, current));
    let executed = expect_success!(ExecutedInvalidationBatch::execute(ready, |work| {
        Ok(work.first().target())
    }));

    assert_eq!(executed.outcome(), &target);
    assert_eq!(executed.binding().axes().target, target);
}

#[test]
fn origin_specific_admission_rejects_cross_origin_work() {
    let target = node(3);
    let revision = DependencyRevision(4);
    let source = source_batch(target, revision);
    let dependency = dependency_batch(target, revision);
    let source_axes = source_binding(target, revision);
    let dependency_axes = dependency_binding(target, revision);

    assert!(matches!(
        AdmittedSourceRecompute::admit(dependency.clone(), dependency_axes.clone()),
        worth_proof::TransitionOutcome::Denied(_)
    ));
    assert!(matches!(
        AdmittedDependencyRecompute::admit(source.clone(), source_axes.clone()),
        worth_proof::TransitionOutcome::Denied(_)
    ));
    assert!(matches!(
        AdmittedStructuralRecompute::admit(source, source_axes),
        worth_proof::TransitionOutcome::Denied(_)
    ));
    assert!(matches!(
        AdmittedDependencyRecompute::admit(dependency, dependency_axes),
        worth_proof::TransitionOutcome::Success(_)
    ));
}

#[test]
fn actual_dependency_and_structural_origins_resolve_without_shared_authority() {
    let target = node(3);
    let revision = DependencyRevision(4);
    let dependency = expect_success!(AdmittedDependencyRecompute::admit(
        dependency_batch(target, revision),
        dependency_binding(target, revision),
    ));
    let structural_batch = InvalidationWorkBatch::single(InvalidationWorkItem::new(
        target,
        revision,
        CanonicalDependencyCauseSet::structural(revision),
    ));
    let structural = expect_success!(AdmittedStructuralRecompute::admit(
        structural_batch,
        InvalidationOriginBindingAxes {
            graph_instance: 1,
            target,
            dependency_revision: revision,
            origin: InvalidationOriginBinding::StructuralMutation {
                ordinal: revision.0,
            },
        },
    ));

    for resolved in [
        ResolvedInvalidationWork::from_dependency(dependency),
        ResolvedInvalidationWork::from_structural(structural),
    ] {
        let lowered = LoweredInvalidationBatch::lower(
            resolved,
            InvalidationReadinessEpoch(6),
            InvalidationStageOrder { stage: 7, order: 8 },
        );
        let current = lowered.binding().clone();
        let ready = expect_success!(ReadyInvalidationBatch::admit(lowered, current));
        let executed = expect_success!(ExecutedInvalidationBatch::execute(ready, |_| Ok(())));
        assert_eq!(executed.outcome(), &());
    }
}

#[test]
fn origin_admission_rejects_forged_payload_binding_axes() {
    let target = node(3);
    let revision = DependencyRevision(4);
    let mut wrong_target = source_binding(target, revision);
    wrong_target.target = node(9);
    assert!(matches!(
        AdmittedSourceRecompute::admit(source_batch(target, revision), wrong_target),
        worth_proof::TransitionOutcome::Denied(_)
    ));

    let mut wrong_generation = source_binding(target, revision);
    wrong_generation.origin = InvalidationOriginBinding::SourceAdmission { generation: 99 };
    assert!(matches!(
        AdmittedSourceRecompute::admit(source_batch(target, revision), wrong_generation),
        worth_proof::TransitionOutcome::Denied(_)
    ));

    let mut wrong_ordinal = dependency_binding(target, revision);
    wrong_ordinal.origin = InvalidationOriginBinding::DependencyCommit {
        cause_set: PendingCauseSetId::EMPTY,
        producer_commit_ordinals: vec![OutputCommitOrdinal(99)],
    };
    assert!(matches!(
        AdmittedDependencyRecompute::admit(dependency_batch(target, revision), wrong_ordinal),
        worth_proof::TransitionOutcome::Denied(_)
    ));
}

#[test]
fn owner_specific_authority_phases_are_not_clone() {
    for (phase, source) in [
        ("AdmittedSourceRecompute", include_str!("source.rs")),
        ("PreparedDirectInvalidation", include_str!("prepared.rs")),
        ("CommittedDirectInvalidation", include_str!("committed.rs")),
        ("AdmittedStructuralRecompute", include_str!("structural.rs")),
        ("ResolvedInvalidationWork", include_str!("resolved.rs")),
        ("LoweredInvalidationBatch", include_str!("lowered.rs")),
        ("ReadyInvalidationBatch", include_str!("ready.rs")),
        ("ExecutedInvalidationBatch", include_str!("executed.rs")),
    ] {
        let declaration = format!("struct {phase}");
        let offset = source.find(&declaration).expect("phase declaration exists");
        let derive_window = &source[offset.saturating_sub(80)..offset];
        assert!(
            !derive_window.contains("Clone"),
            "authority phase must be consumed, not cloned: {phase}"
        );
    }
}
