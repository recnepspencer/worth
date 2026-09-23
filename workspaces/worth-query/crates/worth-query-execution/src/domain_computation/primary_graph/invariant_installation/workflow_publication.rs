//! Built-in custody rule accompanying every workflow schema installation.

use worth_relational::facade::runtime::CustomInvariantRegistration;

use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

use super::{
    denial, WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn registration(
    layout: &WorthQueryWorkflowLayout,
) -> Result<CustomInvariantRegistration, WorthQueryPrimaryGraphInstallationDenial> {
    crate::domain_computation::primary_graph::workflow::schema::publication_immutability_registration(
        layout,
    )
    .map_err(|detail| {
        denial(
            WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected,
            detail,
        )
    })
}
