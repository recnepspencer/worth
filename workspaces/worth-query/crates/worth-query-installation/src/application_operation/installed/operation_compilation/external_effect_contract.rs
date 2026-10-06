use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::application_aftermath::InstalledExternalEffectContract;
use crate::application_schema::{install_inbound_occurrence, InboundOccurrenceInstallationDenial};
use crate::package::WorthQueryPortableExternalEffectContractRecord;

pub(super) fn install_portable_external_effect(
    members: &[ApplicationSchemaMember],
    operation: &str,
    portable: Option<&WorthQueryPortableExternalEffectContractRecord>,
) -> Result<InstalledExternalEffectContract, InboundOccurrenceInstallationDenial> {
    match portable {
        None if members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::OperationInboundOccurrence { operation: declared, .. }
                    if declared == operation
            )
        }) =>
        {
            Err(InboundOccurrenceInstallationDenial::Orphaned)
        }
        None => Ok(InstalledExternalEffectContract::None),
        Some(portable) => Ok(InstalledExternalEffectContract::Declared {
            correlation_family: portable.correlation_family().clone(),
            effect: portable.effect().to_owned(),
            rust_payload_type: portable.payload_type().clone(),
            protocol: portable.protocol().clone(),
            maximum_payload_bytes: portable.maximum_payload_bytes(),
            inbound: install_inbound_occurrence(members, operation, portable.effect())?,
        }),
    }
}
