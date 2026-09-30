use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use super::InstalledInboundOccurrenceContract;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InboundOccurrenceInstallationDenial {
    Duplicate,
    Orphaned,
    InvalidLimits,
    InvalidSource,
}

/// Resolve immutable support from the operation's own schema members. There
/// is no independent registry and an old external effect resolves to `None`.
pub(crate) fn install_inbound_occurrence(
    members: &[ApplicationSchemaMember],
    operation: &str,
    effect: &str,
) -> Result<Option<InstalledInboundOccurrenceContract>, InboundOccurrenceInstallationDenial> {
    let mut resolved = None;
    for member in members {
        let ApplicationSchemaMember::OperationInboundOccurrence {
            operation: declared_operation,
            effect: declared_effect,
            protocol,
            source_identity,
            limits,
        } = member
        else {
            continue;
        };
        if declared_operation != operation {
            continue;
        }
        if resolved.is_some() {
            return Err(InboundOccurrenceInstallationDenial::Duplicate);
        }
        if declared_effect != effect {
            return Err(InboundOccurrenceInstallationDenial::Orphaned);
        }
        if source_identity.trim().is_empty() {
            return Err(InboundOccurrenceInstallationDenial::InvalidSource);
        }
        if !limits.accommodates_payload() {
            return Err(InboundOccurrenceInstallationDenial::InvalidLimits);
        }
        resolved = Some(InstalledInboundOccurrenceContract::new(
            effect.to_owned(),
            protocol.clone(),
            source_identity.clone(),
            *limits,
        ));
    }
    Ok(resolved)
}
