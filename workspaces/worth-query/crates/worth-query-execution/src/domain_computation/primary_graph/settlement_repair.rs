/// Why recovering a deferred settlement did not return a commit receipt.
///
/// The commit was already performed; recovery only completes its settlement.
#[derive(Debug)]
pub enum WorthQueryApplicationSettlementRecoveryError {
    /// The execution owner refused settlement publication before effects.
    ExecutionDenied(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// Relational could not make the settlement durable.
    Durability(worth_relational::facade::publication::DeferredPublicationSettlementError),
    /// The settlement does not match its performed publication, or publication
    /// could not resume.
    Publication(&'static str),
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The settlement completed but no matching idempotency record was found; the
    /// outcome is indeterminate.
    IdempotencyAbsent,
    /// The settlement completed but the idempotency record does not match; the
    /// outcome is indeterminate.
    IdempotencyDrift,
}

impl<Schema> super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub fn recover_deferred_application_settlement(
        &self,
        deferred: &super::WorthQueryApplicationSettlementDeferred,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        WorthQueryApplicationSettlementRecoveryError,
    > {
        self.primary_provider.recover_application_settlement(
            deferred.settlement(),
            deferred.branch(),
            deferred.product_affinity(),
            deferred.idempotency_binding(),
        )
    }
}
