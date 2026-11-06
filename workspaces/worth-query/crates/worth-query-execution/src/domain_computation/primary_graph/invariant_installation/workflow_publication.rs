//! Built-in custody rule accompanying every workflow schema installation.

use worth_relational::facade::runtime::CustomInvariantRegistration;

use crate::domain_computation::graph_layout::workflow::WorthQueryWorkflowLayout;

use {
    super::denial,
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial,
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn registration(
    layout: &WorthQueryWorkflowLayout,
) -> Result<CustomInvariantRegistration, WorthQueryPrimaryGraphInstallationDenial> {
    crate::domain_computation::graph_layout::workflow::fact_custody_registration(layout).map_err(
        |detail| {
            denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
                detail,
            )
        },
    )
}
