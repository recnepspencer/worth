//! Resolves one operation's zero-or-one external-effect contract.

use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::application_aftermath::InstalledExternalEffectContract;

use super::WorthQueryOperationContractCardinalityDenial;

pub(crate) fn operation_external_effect(
    members: &[ApplicationSchemaMember],
    operation: &str,
) -> Result<InstalledExternalEffectContract, WorthQueryOperationContractCardinalityDenial> {
    let mut resolved = None;
    for member in members {
        let ApplicationSchemaMember::OperationExternalEffect {
            operation: installed,
            correlation_family,
            effect,
            rust_payload_type,
            protocol,
            maximum_payload_bytes,
        } = member
        else {
            continue;
        };
        if installed != operation {
            continue;
        }
        if resolved.is_some() {
            return Err(WorthQueryOperationContractCardinalityDenial::AmbiguousExternalEffect);
        }
        resolved = Some(InstalledExternalEffectContract::Declared {
            correlation_family: correlation_family.clone(),
            effect: effect.clone(),
            rust_payload_type: rust_payload_type.clone(),
            protocol: protocol.clone(),
            maximum_payload_bytes: *maximum_payload_bytes,
            inbound: None,
        });
    }
    Ok(resolved.unwrap_or(InstalledExternalEffectContract::None))
}
