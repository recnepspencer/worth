//! Query admission for the actual selected Bridge truth-basis construction.

use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

impl WorthQueryConditionalTruthBasis {
    pub(in crate::domain_computation::primary_graph) fn from_selected_admitted<Schema>(
        selected: crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<Self, Stop> {
        // The selected snapshot and Bridge source are shallow Arc retains.
        // The primary branch projection builds one raw Arc, one framed String,
        // and one framed Arc before the existing truth-basis constructor.

        Ok(Self::from_selected(selected))
    }
}
