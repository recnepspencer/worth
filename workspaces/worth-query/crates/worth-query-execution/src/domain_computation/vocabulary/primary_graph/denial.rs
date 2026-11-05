//! Moved whole from `domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::denial`: the items below and every impl they own.

/// Why this runtime could not admit one runtime-affine physical attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchAttemptDenial {
    /// The committed outbox was observed on another Relational runtime.
    ForeignRelationalRuntime,
    /// The committed publication belongs to another product world.
    ForeignProductWorld,
    /// The outbox commit and its product publication name different commits.
    PublicationCommitMismatch,
    /// An inbound-completing effect carries no co-committed operation slot,
    /// so its completion could never be found by owner maintenance.
    InboundOperationSlotMissing,
    /// This runtime has no physical attempt identities left.
    AttemptIdentityExhausted,
    /// The outstanding-dispatch owner holds no entry for this effect.
    OutstandingDispatchMissing,
    /// The original dispatch's publication has not settled yet.
    OriginalPublicationPending,
    /// The outstanding-dispatch entry disagrees with the committed original.
    OutstandingDispatchMismatch,
    /// The installed limit on concurrent physical sends for this effect is
    /// reached; it frees when an in-flight send settles.
    InFlightCapacityExhausted,
}
