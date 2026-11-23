//! The orphan backstop (PR C C4 r5, plan §1.6). One commit per stage derives
//! only what some live owner selects. At the final drive's quiescence every
//! task that derived in this edit (done, or done before and reopened since)
//! must still be reached from a root through current prerequisites;
//! one that is not was selected and then dropped after it derived, and the
//! edit refuses `SupersededSelectionDropped` rather than publish it. The
//! reopen check refuses the drops it sees first; this catches any other.
use super::*;
use state::TaskState;

impl<S: ApplicationSchema> SynchronousOutputWave<S> {
    /// Refuse a derived task no seeded root reaches any more. Charged once
    /// per task and per prerequisite edge walked.
    pub(super) fn require_no_orphans(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), SynchronousOutputFailure> {
        let subject = "required occurrence frontier";
        checkpoint(context.execution, subject, self.tasks.len() as u64 + 1)?;
        let prerequisites = (0..self.tasks.len())
            .map(|t| self.tasks[t].prerequisites.iter().map(|&p| self.canonical(p)).collect())
            .collect::<Vec<Vec<usize>>>();
        let roots = (0..self.tasks.len())
            .filter(|&i| self.tasks[i].root)
            .map(|i| self.canonical(i))
            .collect::<Vec<_>>();
        let derived = (0..self.tasks.len())
            .map(|i| matches!(self.tasks[i].state, TaskState::Done { .. }) || self.tasks[i].reopened)
            .collect::<Vec<_>>();
        let orphan = first_orphan(&roots, &derived, &prerequisites, &mut |work| {
            checkpoint(context.execution, subject, work)
        })?;
        let Some(i) = orphan else {
            return Ok(());
        };
        observation::record(|| format!("orphan task={i}"));
        let key = match self.tasks[i].key.clone() {
            Some(key) => key,
            None => self.tasks[i].tree.route(context, candidate)?,
        };
        Err(SynchronousOutputFailure(CoupledSourceReadStop::from(
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SupersededSelectionDropped {
                    source: key.scope,
                },
                format!("{}: a derived output no live owner selects", self.tasks[i].subject),
            ),
        )))
    }
}

/// The first task that derived (`derived`) yet is unreached from `roots`
/// through `prerequisites` (canonical indices), charging `charge` per task
/// visited with its edges. Roots only seed the walk: a reopened task is
/// seeded to derive again, but is selected only if some root reaches it.
pub(in super::super) fn first_orphan<E>(
    roots: &[usize],
    derived: &[bool],
    prerequisites: &[Vec<usize>],
    charge: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<Option<usize>, E> {
    let mut reached = vec![false; derived.len()];
    let mut stack = roots.to_vec();
    while let Some(t) = stack.pop() {
        if std::mem::replace(&mut reached[t], true) {
            continue;
        }
        charge(prerequisites[t].len() as u64 + 1)?;
        stack.extend(prerequisites[t].iter().copied());
    }
    Ok((0..derived.len()).find(|&i| derived[i] && !reached[i]))
}
