//! Axis-by-axis proof of the fork-source comparison.
//!
//! `source_is_the_admitted_basis` is the only thing standing between a
//! creation admitted against one source observation and a fork taken from a
//! different one. Its single caller, `observe_exact_fork_source`, turns a
//! `false` here into `RuntimeWorldBranchAdmissionDenial::ForkSourceChanged`
//! before the destination is reserved or any branch is created; the
//! end-to-end proof of that mapping is
//! `relational_fork_creation_compares_a_freshly_observed_source_token`.
//!
//! That end-to-end proof can only move the live axes, and it moves both of
//! them at once. These cases drive the comparison directly with one axis off
//! at a time, so no axis — live or structural — can be dropped from the
//! comparison without a named failure here.

use super::*;

use worth_foundational::facade::{
    FoundationalBranchId, FoundationalBranchReferenceGeneration, FoundationalBranchTarget,
};

const ADMITTED_BRANCH: &str = "admitted-source";
const ADMITTED_INSTANCE: u64 = 7;
const ADMITTED_GENERATION: u64 = 3;
const ADMITTED_TRUTH: u64 = 11;

/// One axis of the comparison, held by value so a case can borrow it.
struct AxisValues {
    runtime_instance_id: u64,
    source_branch: BranchId,
    observation: RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
}

impl AxisValues {
    /// The source basis every case is admitted against.
    fn admitted() -> Self {
        Self {
            runtime_instance_id: ADMITTED_INSTANCE,
            source_branch: BranchId(ADMITTED_BRANCH.to_owned()),
            observation: observation_at(ADMITTED_BRANCH, ADMITTED_GENERATION),
            truth_version: RelationalBranchVersion::new(ADMITTED_TRUTH),
        }
    }

    fn axes(&self) -> ForkSourceAxes<'_> {
        ForkSourceAxes {
            runtime_instance_id: self.runtime_instance_id,
            source_branch: &self.source_branch,
            observation: &self.observation,
            truth_version: self.truth_version,
        }
    }
}

fn observation_at(branch: &str, generation: u64) -> RelationalBranchReferenceObservation {
    RelationalBranchReferenceObservation::new(
        FoundationalBranchId::new(branch).expect("a non-empty branch id is constructible"),
        FoundationalBranchTarget::Empty,
        FoundationalBranchReferenceGeneration::new(generation),
    )
}

/// The one-axis-off table. Each row differs from `AxisValues::admitted()` on
/// exactly the named axis and agrees with it on the other three.
enum OffAxis {
    RuntimeInstance,
    SourceBranch,
    Observation,
    TruthVersion,
}

fn one_axis_off(axis: &OffAxis) -> AxisValues {
    match axis {
        OffAxis::RuntimeInstance => AxisValues {
            runtime_instance_id: ADMITTED_INSTANCE + 1,
            ..AxisValues::admitted()
        },
        OffAxis::SourceBranch => AxisValues {
            source_branch: BranchId("some-other-source".to_owned()),
            ..AxisValues::admitted()
        },
        OffAxis::Observation => AxisValues {
            observation: observation_at(ADMITTED_BRANCH, ADMITTED_GENERATION + 1),
            ..AxisValues::admitted()
        },
        OffAxis::TruthVersion => AxisValues {
            truth_version: RelationalBranchVersion::new(ADMITTED_TRUTH + 1),
            ..AxisValues::admitted()
        },
    }
}

/// Drive one row: the observed source differs on exactly this axis, so the
/// comparison must refuse it and the caller must deny `ForkSourceChanged`.
fn assert_axis_is_compared(axis: &OffAxis, name: &str) {
    let admitted = AxisValues::admitted();
    let observed = one_axis_off(axis);
    assert!(
        !source_is_the_admitted_basis(&observed.axes(), &admitted.axes()),
        "a source differing only on {name} is not the admitted basis and must deny ForkSourceChanged"
    );
}

#[test]
fn a_fork_source_equal_on_every_axis_is_the_admitted_basis() {
    let admitted = AxisValues::admitted();
    let observed = AxisValues::admitted();
    assert!(
        source_is_the_admitted_basis(&observed.axes(), &admitted.axes()),
        "an unmoved source is the admitted basis and must be forked, not denied"
    );
}

#[test]
fn a_fork_source_from_another_runtime_instance_is_not_the_admitted_basis() {
    assert_axis_is_compared(&OffAxis::RuntimeInstance, "runtime_instance_id");
}

#[test]
fn a_fork_source_naming_another_branch_is_not_the_admitted_basis() {
    assert_axis_is_compared(&OffAxis::SourceBranch, "source_branch");
}

#[test]
fn a_fork_source_at_another_observation_is_not_the_admitted_basis() {
    assert_axis_is_compared(&OffAxis::Observation, "observation");
}

#[test]
fn a_fork_source_at_another_truth_version_is_not_the_admitted_basis() {
    assert_axis_is_compared(&OffAxis::TruthVersion, "truth_version");
}
