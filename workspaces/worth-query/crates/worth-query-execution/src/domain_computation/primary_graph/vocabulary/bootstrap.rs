//! `bootstrap`'s vocabulary, moved whole from `primary_graph::bootstrap`: its root file's items here, one child per other source file.
pub(crate) mod publication;

use crate::domain_computation::vocabulary::primary_graph::principal_binding::WorthQueryPrimaryPrincipalBindingLayout;
use worth_query_installation::facade::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::WorthQueryPrincipalMappingStatus;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPrincipalBootstrapRow {
    pub(in crate::domain_computation::primary_graph) binding: String,
    pub(in crate::domain_computation::primary_graph) principal_key: String,
    pub(in crate::domain_computation::primary_graph) principal_identity:
        worth_foundational::facade::AspectValue,
    pub(in crate::domain_computation::primary_graph) identity: WorthQueryExternalPrincipalIdentity,
    pub(in crate::domain_computation::primary_graph) status: WorthQueryPrincipalMappingStatus,
    pub(in crate::domain_computation::primary_graph) layout:
        WorthQueryPrimaryPrincipalBindingLayout,
}
