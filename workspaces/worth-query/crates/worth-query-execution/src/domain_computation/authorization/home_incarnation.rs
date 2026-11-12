//! Preserve the typed stale cause before the existing receiver-affinity checks.
use super::{
    WorthQueryOperationAuthorizationDenial as Denial,
    WorthQueryOperationAuthorizationDenialKind as Kind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

/// This is a stale-incarnation guard, not an admission witness. Foreign owners
/// still pass through each receiver's existing runtime/schema/authority checks.
pub(in crate::domain_computation) fn refuse_stale_home_authority(
    current: RuntimeWorldOwnerIdentity,
    presented: RuntimeWorldOwnerIdentity,
    subject: &str,
) -> Result<(), Denial> {
    match current.admit_incarnation(presented) {
        Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => Err(Denial::new(
            Kind::Refused(Refusal::StaleIncarnation),
            subject,
        )),
        Ok(()) | Err(worth_proof::HomeIncarnationDenial::ForeignHome) => Ok(()),
    }
}
