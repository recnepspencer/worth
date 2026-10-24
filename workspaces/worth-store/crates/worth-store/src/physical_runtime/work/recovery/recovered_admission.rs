use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{
    integrity_admission::{admit_bounded_obligation, scope_from_pending_name},
    observation::PhysicalWorkRecoveryAdmissionCounters,
    PhysicalWorkRecoveryLocator,
};

/// Admits one obligation record a dead runtime left, through the same
/// integrity admission the serving inventory uses. `None` when the name or
/// the record does not admit; such evidence is never interpreted further.
pub(in crate::physical_runtime) fn admit_recovered_obligation(
    store: StableStoreIdentity,
    name: &str,
    record: &[u8],
) -> Option<PhysicalWorkRecoveryLocator> {
    let mut counters = PhysicalWorkRecoveryAdmissionCounters::default();
    let scope = scope_from_pending_name(store, name).ok()?;
    admit_bounded_obligation(scope, record, &mut counters).ok()
}
