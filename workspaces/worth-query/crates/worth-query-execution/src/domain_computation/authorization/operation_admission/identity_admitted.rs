//! One invocation identity and its exact retained resource label.

use std::{fmt::Write, sync::Arc};

use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{WorthQueryOperationAdmissionIdentity, RESOURCE_LABEL};
use crate::domain_computation::primary_graph::InvalidationEditAdmission;

pub(super) enum AdmittedOperationIdentityStop {
    Admission(CompanionPreflightStop),
    Exhausted,
    AccountingOverflow,
}

pub(super) fn mint_operation_identity_admitted(
    admission: &mut InvalidationEditAdmission,
) -> Result<(WorthQueryOperationAdmissionIdentity, Arc<str>), AdmittedOperationIdentityStop> {
    use AdmittedOperationIdentityStop as Stop;
    admission.charge_external_work(3).map_err(Stop::Admission)?;
    let identity = WorthQueryOperationAdmissionIdentity::mint().ok_or(Stop::Exhausted)?;
    let digits = usize::try_from(identity.0.ilog10())
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or(Stop::AccountingOverflow)?;
    let length = RESOURCE_LABEL
        .len()
        .checked_add(digits)
        .ok_or(Stop::AccountingOverflow)?;
    let arc_bytes = crate::domain_computation::arc_str_layout::backing_bytes(length)
        .ok_or(Stop::AccountingOverflow)?;
    // The temporary exact-capacity String and final Arc allocation overlap.
    // The numeric Display path renders at most twenty digits into a stack
    // buffer before writing them to the String. Charge that rendering pass,
    // the complete label write, the Arc copy and its initialized header.
    let work = length
        .checked_mul(2)
        .and_then(|n| n.checked_add(digits))
        .and_then(|n| n.checked_add(20))
        .and_then(|n| {
            n.checked_add(crate::domain_computation::arc_str_layout::initialized_header_work())
        })
        .and_then(|n| n.checked_add(3))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Stop::AccountingOverflow)?;
    let bytes = length
        .checked_add(arc_bytes)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Stop::AccountingOverflow)?;
    admission
        .admit_read_scratch(bytes)
        .and_then(|()| admission.charge_external_work(work))
        .map_err(Stop::Admission)?;
    let mut text = String::with_capacity(length);
    write!(&mut text, "{RESOURCE_LABEL}{}", identity.0).map_err(|_| Stop::AccountingOverflow)?;
    debug_assert_eq!(text.len(), length);
    Ok((identity, Arc::from(text.as_str())))
}
