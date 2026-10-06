use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    application_query::WorthQueryObservedSourceSelection,
};

/// The exact prepared input of a performed producer output. Its source
/// selection retains runtime-owned meaning; a row restored from a checkpoint
/// has no such proof, and a republished output continues its own.
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

    /// The prepared input a republished output continues: the same selected
    /// membership, input and installed producer that performed it.
    pub(in crate::domain_computation::primary_graph) fn continued_by_republication(&self) -> Self {
        Self {
            selection: self.selection.clone(),
            input_identity: self.input_identity,
            installed_edition: self.installed_edition,
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
