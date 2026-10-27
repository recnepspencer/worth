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
    let disclosure = crate::application_operation::source_disclosure::resolve_query(
        declaration.disclosure(),
        queries,
    )
    .ok_or_else(denied)?;
    if disclosure.scope().field() != source.scope().field() {
        return Err(denied());
    }
    if declaration.source() != source.description()
        || declaration.inverse() != inverse.description()
        || declaration.source_handler() != source.handler().identity()
        || declaration.inverse_handler() != inverse.handler().identity()
        || declaration.source_query() != source.source_query_identifier()
        || declaration.inverse_query() != inverse.source_query_identifier()
        || declaration.identity().as_str().trim().is_empty()
        || declaration
            .reapplication_source()
            .is_some_and(|binding| binding != source.description().binding_identity())
    {
        return Err(denied());
    }
    match contract.authority() {
        DeclaredCorrectionAuthority::RuntimeAlone
            if contract.reconciliation().is_none()
                && declaration.completed_external_reconciliation().is_none() => {}
        DeclaredCorrectionAuthority::RuntimeWithExternalOwner
            if contract.reconciliation().is_some()
                && declaration.completed_external_reconciliation() == contract.reconciliation()
                && matches!(
                    contract.mechanism(),
                    Some(PortableCorrectionMechanism::Compensation(_))
                ) => {}
        _ => return Err(denied()),
    }
    match contract.mechanism() {
        Some(PortableCorrectionMechanism::RecordedInverse(recorded))
            if recorded.inverse_operation_slot() == inverse.operation_name() => {}
        Some(PortableCorrectionMechanism::Compensation(compensation))
            if compensation.compensating_operation_slot() == inverse.operation_name() => {}
        _ => return Err(denied()),
    }
    source_outputs::validate(members, source, declaration.source_output_fields())
        .map_err(|()| denied())?;
    authored_identity::validate(members, source, inverse, declaration, contract.mechanism())
        .map_err(|()| denied())?;
    Ok(Some(InstalledCorrectionBinding {
        declaration: declaration.clone(),
        source_type: source.binding_type(),
        inverse_type: inverse.binding_type(),
        disclosure_query_type: disclosure.query_type(),
    }))
}

mod authored_identity;
mod source_outputs;
