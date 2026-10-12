use super::*;

pub(super) fn validate_approval_lifecycle<
    Schema,
    Principal,
    Identity,
    Capability,
    Operation,
    Input,
>(
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let Some(approved) = prepared.approved() else {
        return Ok(());
    };
    let capability = prepared.capability();
    if !approved.belongs_to_lifecycle(
        prepared.runtime().runtime.authority_identity(),
        prepared.graph_work().branch().relational(),
        *capability.identity().bytes(),
        prepared.installed().capability_authority_identity(),
    ) {
        return Err(denial::authorization_denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected,
            capability.contract().name(),
        ));
    }
    Ok(())
}

pub(super) fn validate_approval_support<Schema, Principal, Identity, Capability, Operation, Input>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    if let Some(approved) = prepared.approved() {
        if !approved
            .support_remains_current_in(
                execution,
                relational,
                snapshot,
                prepared.runtime().authorization.bridge(),
            )
            .map_err(|cause| {
                crate::domain_computation::authorization::denial::relational_read_denial(
                    cause,
                    "elevation support currentness",
                )
            })?
        {
            return Err(denial::authorization_denial(
                WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
                prepared.capability().contract().name(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_approved_use<Schema, Principal, Identity, Capability, Operation, Input>(
    prepared: &PreparedCapabilityAdmission<
        '_,
        Schema,
        Principal,
        Identity,
        Capability,
        Operation,
        Input,
    >,
    observed: &RelationalCapabilityObservation<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let Some(approved) = prepared.approved() else {
        return Ok(());
    };
    let elevation = observed.resolved.elevation().ok_or_else(|| {
        denial::authorization_denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationProjectionRejected,
            prepared.capability().contract().name(),
        )
    })?;
    let admitted = approved.admits_active_use(
        WorthQueryRuntimeAuthorityIdentity::clone(&prepared.runtime().runtime.authority_identity()),
        prepared.graph_work().branch().relational(),
        *prepared.capability().identity().bytes(),
        prepared.installed().capability_authority_identity(),
        &observed.revalidation,
        elevation,
        observed.observed.grant(),
    );
    if !admitted {
        return Err(denial::authorization_denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected,
            prepared.capability().contract().name(),
        ));
    }
    Ok(())
}
