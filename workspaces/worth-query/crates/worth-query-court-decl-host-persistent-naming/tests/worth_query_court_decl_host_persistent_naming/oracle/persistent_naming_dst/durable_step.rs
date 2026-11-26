//! The reopen half of one durable DST step: after the world answered the
//! step, crash the Directory Home, reopen the directory as a fresh
//! installation and require the same oracle answers from it.
//!
//! A Directory Home holds a Native naming refusal met while completing its
//! durable candidate as `ProductPreparationPending` recovery custody, which
//! the documented recovery settles by ending the Home and reopening the same
//! location; such a step already reopened before it was observed.
use std::collections::BTreeSet;

use super::candidate;
use super::driver::Driver;
use super::observe::Observed;
use super::ops::Op;
use super::reference::Oracle;
use super::trace::Trace;
use super::{agree, Coverage};

/// Whether `op` must refuse typed, leaving pending recovery custody on a
/// Directory Home that a reopen settles.
pub fn settles_by_reopen(op: &Op) -> bool {
    match op {
        Op::Duplicate { .. } | Op::Unnamed { .. } | Op::CoupledDuplicate { .. } => true,
        Op::Candidate { expected, .. } => candidate::is_refusal(expected),
        _ => false,
    }
}

/// Reopens after `step` (unless its refusal already did) and checks the
/// reopened world against the oracle, tracing the reopen.
#[allow(clippy::too_many_arguments)]
pub fn reopen_after(
    driver: &mut Driver,
    model: &Oracle,
    born: &BTreeSet<usize>,
    observed: &Observed,
    step: usize,
    already_reopened: bool,
    trace: &mut Trace,
    coverage: &mut Coverage,
) -> Result<(), String> {
    coverage.reopens += 1;
    if already_reopened {
        trace.reopen(false, observed, born, driver.naming_commits);
        return Ok(());
    }
    let checkpointed = driver.reopen(step % 2 == 1);
    coverage.checkpointed_reopens += usize::from(checkpointed);
    let observed = driver.observe(step)?;
    agree(model, &observed, born)?;
    trace.reopen(checkpointed, &observed, born, driver.naming_commits);
    Ok(())
}
