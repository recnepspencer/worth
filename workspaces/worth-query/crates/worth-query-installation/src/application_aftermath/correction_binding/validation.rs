use super::InstalledCorrectionBinding;
use crate::application_operation::{
    WorthQueryApplicationOperationInstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind as DenialKind,
};
use std::collections::BTreeMap;
use worth_query_declaration::facade::{
    application_aftermath::{DeclaredCorrectionAuthority, PortableCorrectionMechanism},
    application_operation::ApplicationMutationBindingDescriptor,
    application_query::ApplicationQueryBindingDescriptor,
    application_schema::ApplicationSchemaMember,
};

pub(crate) fn resolve_correction_binding(
    members: &[ApplicationSchemaMember],
    source: &ApplicationMutationBindingDescriptor,
    bindings: &BTreeMap<String, &ApplicationMutationBindingDescriptor>,
    queries: &[ApplicationQueryBindingDescriptor],
) -> Result<Option<InstalledCorrectionBinding>, WorthQueryApplicationOperationInstallationDenial> {
    let contract = members.iter().find_map(|member| match member {
        ApplicationSchemaMember::OperationAftermath {
            operation,
            contract,
        } if operation == source.operation_name() => Some(contract),
        _ => None,
    });
    let Some(contract) = contract else {
        return Ok(None);
    };
    let Some(declaration) = contract.correction_binding() else {
        return Ok(None);
    };
    let denied = || {
        WorthQueryApplicationOperationInstallationDenial::new(
            DenialKind::AftermathInstallationDenied,
            source.operation_name(),
        )
    };
    let inverse = bindings
        .get(declaration.inverse().binding_identity().as_str())
        .ok_or_else(denied)?;
    let disclosure = queries
        .iter()
        .find(|query| query.identity() == declaration.disclosure().parts().binding.as_str())
        .ok_or_else(denied)?;
    if !declaration.disclosure().matches_descriptor(disclosure)
        || disclosure.scope().field() != source.scope().field()
    {
        return Err(denied());
    }
    if declaration.source() != source.description()
        || declaration.inverse() != inverse.description()
        || declaration.source_handler() != source.handler().identity()
        || declaration.inverse_handler() != inverse.handler().identity()
        || declaration.source_query() != source.source_query_identifier()
        || declaration.inverse_query() != inverse.source_query_identifier()
        || declaration.identity().as_str().trim().is_empty()
        || contract.authority() != DeclaredCorrectionAuthority::RuntimeAlone
    {
        return Err(denied());
    }
    match contract.mechanism() {
        Some(PortableCorrectionMechanism::RecordedInverse(recorded))
            if recorded.inverse_operation_slot() == inverse.operation_name() => {}
        Some(PortableCorrectionMechanism::Compensation(compensation))
            if compensation.compensating_operation_slot() == inverse.operation_name() => {}
        _ => return Err(denied()),
    }
    Ok(Some(InstalledCorrectionBinding {
        declaration: declaration.clone(),
        source_type: source.binding_type(),
        inverse_type: inverse.binding_type(),
        disclosure_query_type: disclosure.query_type(),
    }))
}
