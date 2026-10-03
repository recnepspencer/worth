use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};

use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    application_query::WorthQueryObservedSourceSelection,
};

/// The exact prepared input of a performed producer output. Its source
/// selection retains runtime-owned meaning; restored rows have no such proof.
pub(in crate::domain_computation::primary_graph) struct PreparedInputReuseKey {
    selection: WorthQueryObservedSourceSelection,
    input_identity: [u8; 32],
    installed_edition: InstalledProducerEdition,
    selected_program: Option<(ApplicationProgramIdentity, ApplicationProgramRevision)>,
}

impl PreparedInputReuseKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        selection: WorthQueryObservedSourceSelection,
        input_identity: [u8; 32],
        installed_edition: InstalledProducerEdition,
        selected_program: Option<(ApplicationProgramIdentity, ApplicationProgramRevision)>,
    ) -> Self {
        Self {
            selection,
            input_identity,
            installed_edition,
            selected_program,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn same_prepared_input_as<E>(
        &self,
        other: &Self,
        mut admit: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(64)?;
        if self.input_identity != other.input_identity
            || self.installed_edition != other.installed_edition
        {
            return Ok(false);
        }
        match (&self.selected_program, &other.selected_program) {
            (None, None) => {}
            (Some((left, left_revision)), Some((right, right_revision))) => {
                admit(
                    left.as_str()
                        .len()
                        .max(right.as_str().len())
                        .saturating_add(32),
                )?;
                if left != right || left_revision != right_revision {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
        self.selection
            .same_selected_membership_as(&other.selection, admit)
    }
}
