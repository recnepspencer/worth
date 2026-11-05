//! Moved whole from `domain_computation::provider_session::graph_obligation::session_identity`: the items below and every impl they own.
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub(in crate::domain_computation) static NEXT_GRAPH_WORK_SESSION_IDENTITY: AtomicU64 =
    AtomicU64::new(1);

pub(in crate::domain_computation) static NEXT_GRAPH_WORK_MANAGED_RUN_IDENTITY: AtomicU64 =
    AtomicU64::new(1);

/// Fixed-width identity of the managed run that owns one provider session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryGraphWorkManagedRunIdentity(u64);

impl WorthQueryGraphWorkManagedRunIdentity {
    pub(in crate::domain_computation) fn mint() -> Option<Self> {
        {
            let update_counter = &NEXT_GRAPH_WORK_MANAGED_RUN_IDENTITY;
            let mut value = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = value.checked_add(1) else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .ok()
        .map(Self)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Fixed-width identity of one live graph-work session.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryGraphWorkSessionIdentity(u64);

impl WorthQueryGraphWorkSessionIdentity {
    /// Coordinate ownership for a provider invocation under an actual Native
    /// Source loan. The identity itself grants no public purpose or Source read.
    pub(in crate::domain_computation) fn for_native_source(
        scope: &worth_relational::facade::runtime::ManagedIndexSourceScope<'_>,
    ) -> Result<Self, worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial> {
        use worth_relational::facade::runtime::{
            DerivedRelationReadDenial, ManagedIndexSourcePreparationDenial as Denial,
        };
        if !scope.scope_exists()? && !scope.committed_scope_exists()? {
            return Err(Denial::RootUnavailable);
        }
        Self::mint().ok_or_else(|| Denial::Source(DerivedRelationReadDenial::Request(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow,
        )))
    }
    pub(in crate::domain_computation) fn mint() -> Option<Self> {
        {
            let update_counter = &NEXT_GRAPH_WORK_SESSION_IDENTITY;
            let mut value = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = value.checked_add(1) else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .ok()
        .map(Self)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl WorthQueryGraphWorkManagedRunIdentity {
    #[doc(hidden)]
    pub fn mint_for_test() -> Option<Self> {
        Self::mint()
    }
}

impl WorthQueryGraphWorkSessionIdentity {
    #[doc(hidden)]
    pub fn mint_for_test() -> Option<Self> {
        Self::mint()
    }
}
