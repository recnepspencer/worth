//! Moved whole from `domain_computation::primary_graph::application_query::authorized_read`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_query::admission::MetadataValidationDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::read_execution_denial::WorthQueryApplicationReadExecutionDenial;

pub enum WorthQueryAuthorizedApplicationReadDenial {
    StalePrincipal,
    StaleScope,
    StaleBasisScope,
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    Read(WorthQueryApplicationReadExecutionDenial),
    Session,
}

impl From<MetadataValidationDenial>
    for crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial
{
    fn from(cause: MetadataValidationDenial) -> Self {
        match cause {
            MetadataValidationDenial::Session => Self::Session,
            MetadataValidationDenial::Authorization(cause) => Self::Authorization(cause),
        }
    }
}
