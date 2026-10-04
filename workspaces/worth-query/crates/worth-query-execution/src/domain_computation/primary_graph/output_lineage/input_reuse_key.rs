use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    application_query::WorthQueryObservedSourceSelection,
};

/// The exact prepared input of a performed producer output. Its source
/// selection retains runtime-owned meaning; restored rows have no such proof.
/// The installed edition is the producer's whole identity: a runtime installs
/// one producer under it, whichever program selects the occurrence.
pub(in crate::domain_computation::primary_graph) struct PreparedInputReuseKey {
    selection: WorthQueryObservedSourceSelection,
    input_identity: [u8; 32],
    installed_edition: InstalledProducerEdition,
}

impl PreparedInputReuseKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        selection: WorthQueryObservedSourceSelection,
        input_identity: [u8; 32],
        installed_edition: InstalledProducerEdition,
    ) -> Self {
        Self {
            selection,
            input_identity,
            installed_edition,
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
        self.selection
            .same_selected_membership_as(&other.selection, admit)
    }
}
