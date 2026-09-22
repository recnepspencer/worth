//! Which rostered program one attempt's own occurrence is actually running.
//!
//! Support is admitted once, at installation, but activation is owner truth
//! carried through World and may differ between two occurrences of the same
//! host. A program-gated commit therefore resolves the active program by
//! point-reading the branch program activation record at the attempt's exact
//! snapshot, before any effect is bound.
//!
//! The observation is not carried into the attempt's decision read set.
//! Decision-read-set admission validates every retained fact against the
//! families the operation's installed graph contract declares, and the platform
//! activation kind belongs to none of them, so a fact appended here is refused
//! for every program-hosted commit. Recording the read therefore needs a
//! declared platform read family on the graph-work contract, which is owned by
//! the installation surface rather than by this gate.
//!
//! A late activation change is nonetheless fenced, by the product head the
//! attempt already carries. The snapshot read here is the one the attempt's
//! lease acquired at its product observation's own Relational basis, and the
//! publication the attempt ends in expects that same selected occurrence as the
//! product head. Activation is an ordinary branch entity, so changing it is
//! itself a published commit on that branch, and any such commit moves the head
//! World compares. The attempt is then refused `StaleExpectedProductHead`
//! before it can perform. What is missing is diagnostic, not authoritative: the
//! refusal names a moved head rather than the program that moved under it.

use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::WorthQueryProgramSupportEntry;

use super::super::super::program_occurrence::WorthQueryInstalledProgramSupport;
use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationEffectProgram,
    WorthQueryProgramActivationUnresolved,
};

/// The rostered program one occurrence resolved to, together with the exact
/// stored rendering the attempt observed.
pub(super) struct WorthQueryOccurrenceProgram<'support> {
    entry: &'support WorthQueryProgramSupportEntry,
    rendering: AspectValue,
}

impl<'support> WorthQueryOccurrenceProgram<'support> {
    pub(super) const fn entry(&self) -> &'support WorthQueryProgramSupportEntry {
        self.entry
    }

    pub(super) const fn rendering(&self) -> &AspectValue {
        &self.rendering
    }
}

/// Resolves the program active on this attempt's occurrence.
///
/// The read costs one ordinary point read against the attempt's own snapshot
/// and observes the program without changing it. An activation that was never
/// published, cannot be read, or names no rostered program is refused, each
/// under its own named cause: a host that cannot attribute work to admitted
/// meaning must not perform it.
pub(super) fn resolve_occurrence_program<'support, Schema, Operation, Input, Scope>(
    support: &'support WorthQueryInstalledProgramSupport<Schema>,
    program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Result<WorthQueryOccurrenceProgram<'support>, WorthQueryApplicationCommitDenial> {
    let entity_id = support
        .activation()
        .published()
        .ok_or_else(|| unresolved(WorthQueryProgramActivationUnresolved::NeverPublished))?;
    let layout = program.read_set.lease.layout.program_activation().clone();
    let rendering = program
        .read_set
        .lease
        .handle()
        .with_runtime(|runtime| {
            super::super::observe_field_value(
                runtime,
                program.read_set.lease.snapshot(),
                entity_id,
                layout.entity_kind,
                &layout.program_revision_locator,
            )
        })
        .ok_or_else(|| unresolved(WorthQueryProgramActivationUnresolved::Unreadable))?;
    let entry = support
        .rostered_for_rendering(&rendering)
        .ok_or_else(|| unresolved(WorthQueryProgramActivationUnresolved::NamesNoRosteredProgram))?;
    Ok(WorthQueryOccurrenceProgram { entry, rendering })
}

fn unresolved(cause: WorthQueryProgramActivationUnresolved) -> WorthQueryApplicationCommitDenial {
    WorthQueryApplicationCommitDenial::program_activation_unresolved(cause)
}
