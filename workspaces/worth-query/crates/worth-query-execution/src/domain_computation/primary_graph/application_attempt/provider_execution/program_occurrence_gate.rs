//! Which rostered program one attempt's own occurrence is actually running.
//!
//! Support is admitted once, at installation, but activation is owner truth
//! carried through World and may differ between two occurrences of the same
//! host. A program-gated commit therefore resolves the active program by
//! point-reading the branch program activation record at the attempt's exact
//! snapshot, before any effect is bound.
//!
//! Activation is a platform-owned dependency outside the operation's declared
//! graph reads. Ordinary prepared commits may readmit an unchanged decision on
//! a later product head, so readmission compares the activation rendering on the
//! original and newly admitted snapshots before minting provider affinity.
//! A later change is still fenced by the exact Product publication CAS.

use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::WorthQueryProgramSupportEntry;

#[cfg(test)]
mod tests;

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

/// Compares the producer's selected-program binding with this exact occurrence.
pub(super) fn require_selected_program_matches_occurrence(
    occurrence: &WorthQueryOccurrenceProgram<'_>,
    selected_program: Option<(
        &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
        &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    )>,
) -> Result<(), WorthQueryApplicationCommitDenial> {
    if let Some((identity, revision)) = selected_program {
        if occurrence.entry().identity() != identity || occurrence.entry().revision() != revision {
            return Err(
                WorthQueryApplicationCommitDenial::program_revision_not_active_on_occurrence(
                    identity,
                    revision,
                    occurrence.entry().identity(),
                    occurrence.entry().revision(),
                ),
            );
        }
    }
    Ok(())
}

/// Requires the activated program to declare the effectful operation route.
pub(super) fn require_occurrence_acts_through<Operation: 'static>(
    occurrence: &WorthQueryOccurrenceProgram<'_>,
) -> Result<(), WorthQueryApplicationCommitDenial> {
    if !occurrence
        .entry()
        .acts_through_operation(std::any::TypeId::of::<Operation>())
    {
        return Err(
            WorthQueryApplicationCommitDenial::operation_not_declared_by_active_program::<Operation>(
                occurrence.entry().identity(),
                occurrence.entry().revision(),
            ),
        );
    }
    Ok(())
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
    resolve_at_lease(support, &program.read_set.lease)
}

/// Preserve the program selection already checked at the public commit entry.
pub(super) fn require_readmitted_program_matches<Schema>(
    support: &WorthQueryInstalledProgramSupport<Schema>,
    retained: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
    current: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
) -> Result<(), WorthQueryApplicationCommitDenial> {
    let original = resolve_at_lease(support, retained)?;
    let selected = resolve_at_lease(support, current)?;
    if original.rendering() != selected.rendering() {
        return Err(
            WorthQueryApplicationCommitDenial::program_not_active_on_occurrence(
                original.entry().identity(),
                selected.entry().identity(),
                selected.entry().revision(),
            ),
        );
    }
    require_selected_program_matches_occurrence(
        &selected,
        Some((original.entry().identity(), original.entry().revision())),
    )
}

fn resolve_at_lease<'support, Schema>(
    support: &'support WorthQueryInstalledProgramSupport<Schema>,
    lease: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
) -> Result<WorthQueryOccurrenceProgram<'support>, WorthQueryApplicationCommitDenial> {
    let entity_id = support
        .activation()
        .published()
        .ok_or_else(|| unresolved(WorthQueryProgramActivationUnresolved::NeverPublished))?;
    let layout = lease.layout.program_activation().clone();
    let rendering = lease
        .handle()
        .with_runtime(|runtime| {
            super::super::observe_field_value(
                runtime,
                lease.snapshot(),
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
