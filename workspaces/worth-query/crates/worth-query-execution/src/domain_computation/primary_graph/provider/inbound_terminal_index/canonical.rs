//! Exact immutable terminal meaning, derived from actual completion sources.
use super::*;
impl WorthQueryCanonicalInboundCompletion {
    pub(in crate::domain_computation) fn matches_recorded_message(
        &self,
        accepted: &crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundAcceptedMessage,
        home: worth_foundational::facade::HomeIdentity,
    ) -> bool {
        let Some(owner) = self.original_world_commit.owner_identity().durable() else {
            return false;
        };
        self.matches_effect(&accepted.operation, &accepted.claims)
            && owner.home() == home
            && owner.slot().get() == accepted.source.world_slot
            && self.original_world_commit.ordinal() == accepted.source.world_commit
            && self.original_incarnation.ordinal() == accepted.source.incarnation
            && self.original_relational_commit.commit_id.0 == accepted.source.native_commit
            && self.original_relational_commit.version_id.0 == accepted.source.native_version
            && self.original_relational_commit.branch_id.0 == accepted.source.native_branch
    }

    /// A workflow may consume only the terminal for its original committed
    /// operation. Correlation selects this row; these owner facts authorize its
    /// use after both the dispatch and completion have been World-performed.
    pub(in crate::domain_computation::primary_graph) fn matches_original_dispatch(
        &self,
        record: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        original_relational_commit: &RelationalCommitReceipt,
        original_world_commit: &CompositeCommitIdentity,
        original_incarnation: ProductBranchIncarnation,
    ) -> bool {
        let Some(inbound) = record.inbound() else {
            return false;
        };
        self.correlation == *record.correlation()
            && record.operation_slot() == Some(self.operation.as_str())
            && record.effect() == inbound.effect()
            && self.source == inbound.source_identity()
            && self.correlation_family == record.correlation_family().as_str()
            && self.protocol_identity == *record.protocol_identity()
            && inbound.protocol().identity() == record.protocol_identity()
            && self.protocol_version == record.protocol_version()
            && inbound.protocol().version() == record.protocol_version()
            && self.payload == record.payload()
            && self.original_incarnation == original_incarnation
            && &self.original_relational_commit == original_relational_commit
            && &self.original_world_commit == original_world_commit
    }

    pub(in crate::domain_computation::primary_graph) fn from_verified_row(
        row: WorthQueryCanonicalCompletionRow,
        original_incarnation: ProductBranchIncarnation,
        original_world_commit: CompositeCommitIdentity,
        completion_world_commit: CompositeCommitIdentity,
        completion_attempt: CompositePublicationAttemptIdentity,
    ) -> Self {
        Self {
            correlation: row.correlation,
            operation: row.operation,
            audience: row.audience,
            source: row.source,
            key_epoch: row.key_epoch,
            message_identity: row.message_identity,
            signed_meaning_digest: row.signed_meaning_digest,
            provenance: row.provenance,
            correlation_family: row.family,
            protocol_identity: row.protocol_identity,
            protocol_version: row.protocol_version,
            payload: row.payload,
            expires_at_unix_seconds: row.expires_at,
            original_incarnation,
            original_relational_commit: row.original_commit,
            original_world_commit,
            completion_relational_commit: row.completion_commit,
            completion_world_commit,
            completion_attempt,
        }
    }
    pub(super) fn seal(terminal: &WorthQueryInboundTerminalOwnerResult) -> Option<Self> {
        let accepted = terminal.accepted();
        let original = accepted.owner();
        original.record().inbound()?;
        let performed = terminal.publication().publication();
        let settlement = performed.component_results().relational_settlement()?;
        if performed
            .component_results()
            .relational_commit_result()
            .is_none()
            || performed.commit().identity() != performed.new_product_head().selected_commit()
            || performed.new_product_head().lifecycle_incarnation()
                != terminal.original_incarnation()
            || original
                .committed_product_publication()
                .product_incarnation()
                != terminal.original_incarnation()
            || original.committed_product_publication().relational_commit()
                != original.commit_reference()
        {
            return None;
        }
        let claims = accepted.claims();
        Some(Self {
            correlation: *original.record().correlation(),
            operation: accepted.operation().to_owned(),
            audience: claims.audience.clone(),
            source: claims.source_identity.clone(),
            key_epoch: Some(claims.key_epoch),
            message_identity: Some(claims.message_identity),
            signed_meaning_digest: Some(*accepted.signed_meaning_digest()),
            provenance: WorthQueryCompletionProvenance::AuthenticatedInbound,
            correlation_family: claims.correlation_family.clone(),
            protocol_identity: claims.protocol_identity.clone(),
            protocol_version: claims.protocol_version,
            payload: claims.payload.clone(),
            expires_at_unix_seconds: Some(claims.expires_at_unix_seconds),
            original_incarnation: terminal.original_incarnation(),
            original_relational_commit: original.commit_reference().clone(),
            original_world_commit: original
                .committed_product_publication()
                .composite_commit()
                .clone(),
            completion_relational_commit: settlement.clone(),
            completion_world_commit: performed.commit().identity().clone(),
            completion_attempt: performed.attempt_identity().clone(),
        })
    }

    pub(in crate::domain_computation) const fn correlation(
        &self,
    ) -> &ExternalEffectCorrelationIdentity {
        &self.correlation
    }

    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.operation
    }

    pub(in crate::domain_computation) fn matches_effect(
        &self,
        operation: &str,
        claims: &WorthQueryInboundOccurrenceClaims,
    ) -> bool {
        self.operation == operation
            && self.audience == claims.audience
            && self.source == claims.source_identity
            && self.correlation_family == claims.correlation_family
            && self.protocol_identity == claims.protocol_identity
            && self.protocol_version == claims.protocol_version
            && self.payload == claims.payload
    }

    pub(in crate::domain_computation) fn matches_message(
        &self,
        claims: &WorthQueryInboundOccurrenceClaims,
    ) -> bool {
        self.message_identity == Some(claims.message_identity)
            && self.key_epoch == Some(claims.key_epoch)
            && self.signed_meaning_digest == Some(claims.signed_meaning_digest)
    }

    pub(in crate::domain_computation) const fn authenticated_message_identity(
        &self,
    ) -> Option<&[u8; 32]> {
        self.message_identity.as_ref()
    }

    pub(in crate::domain_computation) const fn authenticated_key_epoch(&self) -> Option<u64> {
        self.key_epoch
    }

    pub(in crate::domain_computation) const fn original_world_commit(
        &self,
    ) -> &CompositeCommitIdentity {
        &self.original_world_commit
    }

    pub(in crate::domain_computation) const fn completion_world_commit(
        &self,
    ) -> &CompositeCommitIdentity {
        &self.completion_world_commit
    }

    pub(in crate::domain_computation) const fn completion_attempt(
        &self,
    ) -> &CompositePublicationAttemptIdentity {
        &self.completion_attempt
    }
}
