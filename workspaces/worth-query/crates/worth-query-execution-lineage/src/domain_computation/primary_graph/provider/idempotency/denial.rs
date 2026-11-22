//! Pure provider idempotency resolution denial enum.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderIdempotencyResolutionDenial {
    ForeignAffinity,
    PreparationBusy,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    RecordedIntentUnverifiable,
    ExecutionDenied(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    NativeKeyLookup {
        index: worth_relational::facade::indexes::DerivedIndexId,
        cause: worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind,
    },
    Unavailable,
}

impl From<&'static str> for WorthQueryProviderIdempotencyResolutionDenial {
    fn from(_: &'static str) -> Self {
        Self::Unavailable
    }
}
