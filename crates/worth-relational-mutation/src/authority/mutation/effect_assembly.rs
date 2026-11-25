use crate::authority::mutation::effect_diagnostics::diagnostic_entry_for_mutation_event;
use crate::authority::mutation::effect_publication::record_publication_effect_for_mutation;

use super::outcomes::MutationOutcome;
use super::{MutationEffect, MutationWorkspace};

pub(crate) fn assemble_effect(
    outcome: MutationOutcome,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationEffect, super::MutationApplyDenial> {
    let version_id = workspace.version_id();
    let mut effect = MutationEffect::with_capacity(outcome.changes.len(), outcome.events.len());

    for change in outcome.changes {
        record_publication_effect_for_mutation(&mut effect, change, workspace, version_id)?;
    }

    for event in outcome.events {
        effect
            .diagnostics
            .entries
            .push(diagnostic_entry_for_mutation_event(event));
    }

    Ok(effect)
}
