//! One invocation identity and its exact retained resource label.

use std::fmt::Write;
use std::sync::Arc;

use super::WorthQueryOperationAdmissionIdentity;
use super::RESOURCE_LABEL;

pub(super) enum AdmittedOperationIdentityStop {
    Exhausted,
    AccountingOverflow,
}

pub(super) fn mint_operation_identity_admitted(
) -> Result<(WorthQueryOperationAdmissionIdentity, Arc<str>), AdmittedOperationIdentityStop> {
    use AdmittedOperationIdentityStop as Stop;

    let identity = WorthQueryOperationAdmissionIdentity::mint().ok_or(Stop::Exhausted)?;
    let digits = usize::try_from(identity.0.ilog10())
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or(Stop::AccountingOverflow)?;
    let length = RESOURCE_LABEL
        .len()
        .checked_add(digits)
        .ok_or(Stop::AccountingOverflow)?;

    // The temporary exact-capacity String and final Arc allocation overlap.
    // The numeric Display path renders at most twenty digits into a stack
    // buffer before writing them to the String. Charge that rendering pass,
    // the complete label write, the Arc copy and its initialized header.

    let mut text = String::with_capacity(length);
    write!(&mut text, "{RESOURCE_LABEL}{}", identity.0).map_err(|_| Stop::AccountingOverflow)?;
    debug_assert_eq!(text.len(), length);
    Ok((identity, Arc::from(text.as_str())))
}
