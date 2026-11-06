//! Immutable installed producer contracts, inspected without discovering or running outputs.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;

impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn validate_history_group_output<D: WorthQueryApplicationOutputDemand<S>>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        if self
            .runtime()
            .installed_producers
            .supports_history_group::<Family<S, D>>()
        {
            Ok(())
        } else {
            Err(WorthQueryDocumentHistoryDenial::UnsupportedOperation)
        }
    }
}
