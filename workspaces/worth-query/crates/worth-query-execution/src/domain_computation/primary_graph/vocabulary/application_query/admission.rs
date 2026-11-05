//! Moved whole from `domain_computation::primary_graph::application_query::history::admission`: the items below and every impl they own.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;

pub(in crate::domain_computation::primary_graph) enum MetadataValidationDenial {
    Session,
    Authorization(WorthQueryOperationAuthorizationDenial),
}
