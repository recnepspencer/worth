//! Program binding and original migration-prefix comparison.
use super::comparison::{HostDivergence, ProjectionPart};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use worth_foundational::facade::HomeProgramBinding;
use worth_relational::facade::history::RelationalCommitReceipt;
/// Actual readmitted transition prefix data, not a transition phase proof.
pub struct TransitionProjection {
    pub before: HomeProgramBinding,
    pub completed_steps: Vec<RelationalCommitReceipt>,
    pub published_binding: Option<HomeProgramBinding>,
}
fn program(h: &HomeProgramBinding, r: &HomeProgramBinding) -> Result<(), HostDivergence> {
    if h != r {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::ProgramBinding,
            index: 0,
        });
    }
    Ok(())
}
pub fn compare_transition(
    map: &mut DurableIdentityCorrespondence,
    h: &TransitionProjection,
    r: &TransitionProjection,
) -> Result<(), HostDivergence> {
    program(&h.before, &r.before)?;
    if h.completed_steps.len() != r.completed_steps.len() {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::MigrationSteps,
            index: 0,
        });
    }
    for (h, r) in h.completed_steps.iter().zip(&r.completed_steps) {
        if h.parents.len() != r.parents.len() {
            return Err(HostDivergence::Meaning {
                part: ProjectionPart::MigrationSteps,
                index: 0,
            });
        }
        map.admit(Id::NativeCommit(h.commit_id), Id::NativeCommit(r.commit_id))?;
        map.admit(
            Id::NativeVersion(h.version_id),
            Id::NativeVersion(r.version_id),
        )?;
        map.admit(
            Id::NativeBranch(h.branch_id.clone()),
            Id::NativeBranch(r.branch_id.clone()),
        )?;
        for (h, r) in h.parents.iter().zip(&r.parents) {
            map.admit(Id::NativeCommit(*h), Id::NativeCommit(*r))?;
        }
    }
    match (&h.published_binding, &r.published_binding) {
        (None, None) => Ok(()),
        (Some(h), Some(r)) => program(h, r),
        (None, Some(_)) | (Some(_), None) => Err(HostDivergence::Meaning {
            part: ProjectionPart::ProgramBinding,
            index: 0,
        }),
    }
}
