//! Capture the original revocation once, then retain its native command in the worker.
use super::*;
use crate::http::server::capability_revocation_custody::{
    native_denial, CapabilityRevocationCustody,
};
use bank_domain::{estate::EstateAction, schema::BankSchema};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequest;

pub(super) fn start(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
    controls: BankMutationControls,
    request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>,
) -> Result<(SourceMutationCustody, Option<BankHttpDenial>), BankHttpDenial> {
    let command = runtime
        .retain_estate_capability_revocation(
            principal,
            action,
            controls.idempotency_key().clone(),
            controls.request(),
        )
        .map_err(|(_action, _key, _prepare, cause)| native_denial(cause))?;
    let (owner, refusal) = CapabilityRevocationCustody::Active(command).advance(request);
    Ok((SourceMutationCustody::CapabilityRevocation(owner), refusal))
}
