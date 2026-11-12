//! Immutable accepted message meaning and its recorded delivery disposition.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceClaims;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct QueryInboundAcceptedMessage {
    pub operation: String,
    pub claims: WorthQueryInboundOccurrenceClaims,
    pub envelope_bytes: u64,
    pub source: QueryInboundDispatchSource,
}
impl QueryInboundAcceptedMessage {
    pub(in crate::domain_computation) fn retained_bytes(
        operation: &str,
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope_bytes: u64,
        native_branch: &str,
    ) -> Option<u64> {
        super::codec::accepted_retained_bytes(operation, claims, envelope_bytes, native_branch)
    }
}

/// Descriptive original coordinates, checked against freshly admitted owners.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct QueryInboundDispatchSource {
    pub world_slot: u64,
    pub world_commit: u64,
    pub world_attempt: u64,
    pub incarnation: u64,
    pub native_commit: u64,
    pub native_version: u64,
    pub native_branch: String,
    pub entity: [u64; 3],
}

impl QueryInboundDispatchSource {
    pub(in crate::domain_computation) fn from_owner(
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<Self, QueryDispatchJournalDenial> {
        let publication = owner.committed_product_publication();
        let durable = publication
            .product_branch()
            .owner_identity()
            .durable()
            .ok_or(QueryDispatchJournalDenial::ForeignHome)?;
        let worth_relational::facade::transactions::RecordRef::Entity(entity) = owner.record_ref()
        else {
            return Err(QueryDispatchJournalDenial::MalformedFrame);
        };
        Ok(Self {
            world_slot: durable.slot().get(),
            world_commit: publication.composite_commit().ordinal(),
            world_attempt: publication.publication_attempt().ordinal(),
            incarnation: publication.product_incarnation().ordinal(),
            native_commit: owner.commit_reference().commit_id.0,
            native_version: owner.commit_reference().version_id.0,
            native_branch: owner.commit_reference().branch_id.0.clone(),
            entity: [
                entity.partition_value_u64(),
                entity.local_slot_value(),
                u64::from(entity.generation_value()),
            ],
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct QueryInboundMessageKey {
    pub audience: String,
    pub protocol: String,
    pub source: String,
    pub epoch: u64,
    pub message: [u8; 32],
}
impl QueryInboundMessageKey {
    pub(in crate::domain_computation) fn from_claims(
        claims: &WorthQueryInboundOccurrenceClaims,
    ) -> Self {
        Self {
            audience: claims.audience.clone(),
            protocol: claims.protocol_identity.as_str().to_owned(),
            source: claims.source_identity.clone(),
            epoch: claims.key_epoch,
            message: claims.message_identity,
        }
    }
}

/// Recorded original operation/audience meaning; no verifier or live authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct QueryTransportBinding {
    pub correlation: CanonicalDigestId,
    pub operation: String,
    pub audience: Option<String>,
    pub source: QueryInboundDispatchSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum QueryInboundRecord {
    TransportBinding(QueryTransportBinding),
    Accepted(QueryInboundAcceptedMessage),
    Acknowledged {
        message: QueryInboundMessageKey,
        completion_world: u64,
    },
    Expired {
        message: QueryInboundMessageKey,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct QueryInboundRecordedMessage {
    pub acceptance_position: u64,
    pub accepted: QueryInboundAcceptedMessage,
    pub acknowledgement: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct QueryInboundMessageLedger {
    transport_bindings: BTreeMap<CanonicalDigestId, QueryTransportBinding>,
    messages: BTreeMap<QueryInboundMessageKey, QueryInboundRecordedMessage>,
    by_position: BTreeMap<u64, QueryInboundMessageKey>,
}
impl QueryInboundMessageLedger {
    pub(super) fn validate(
        &self,
        record: &QueryInboundRecord,
    ) -> Result<(), QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial::MalformedFrame;
        match record {
            QueryInboundRecord::TransportBinding(binding) => {
                if let Some(original) = self.transport_bindings.get(&binding.correlation)
                    && (original.operation != binding.operation
                        || original.source != binding.source
                        || (original.audience.is_some() && original.audience != binding.audience))
                {
                    return Err(MalformedFrame);
                }
            }
            QueryInboundRecord::Accepted(accepted) => {
                if self
                    .messages
                    .contains_key(&QueryInboundMessageKey::from_claims(&accepted.claims))
                {
                    return Err(MalformedFrame);
                }
            }
            QueryInboundRecord::Acknowledged {
                message,
                completion_world,
            } => {
                let entry = self.messages.get(message).ok_or(MalformedFrame)?;
                if *completion_world == 0 || entry.acknowledgement.is_some() {
                    return Err(MalformedFrame);
                }
            }
            QueryInboundRecord::Expired { message } => {
                if self
                    .messages
                    .get(message)
                    .is_none_or(|entry| entry.acknowledgement.is_none())
                {
                    return Err(MalformedFrame);
                }
            }
        }
        Ok(())
    }
    pub(super) fn fold(&mut self, record: QueryInboundRecord, position: u64) {
        match record {
            QueryInboundRecord::TransportBinding(binding) => {
                self.transport_bindings.insert(binding.correlation, binding);
            }
            QueryInboundRecord::Accepted(accepted) => {
                let key = QueryInboundMessageKey::from_claims(&accepted.claims);
                self.by_position.insert(position, key.clone());
                self.messages.insert(
                    key,
                    QueryInboundRecordedMessage {
                        acceptance_position: position,
                        accepted,
                        acknowledgement: None,
                    },
                );
            }
            QueryInboundRecord::Acknowledged {
                message,
                completion_world,
            } => {
                self.messages
                    .get_mut(&message)
                    .expect("validated acceptance")
                    .acknowledgement = Some(completion_world);
            }
            QueryInboundRecord::Expired { message } => {
                let removed = self.messages.remove(&message).expect("validated message");
                self.by_position.remove(&removed.acceptance_position);
            }
        }
    }
    pub(super) fn transport_binding(
        &self,
        correlation: &CanonicalDigestId,
    ) -> Option<&QueryTransportBinding> {
        self.transport_bindings.get(correlation)
    }
    pub(super) fn messages(
        &self,
    ) -> &BTreeMap<QueryInboundMessageKey, QueryInboundRecordedMessage> {
        &self.messages
    }
    pub(super) fn next_after(&self, position: u64) -> Option<QueryInboundRecordedMessage> {
        use std::ops::Bound::{Excluded, Unbounded};
        let (_, key) = self
            .by_position
            .range((Excluded(position), Unbounded))
            .next()?;
        self.messages.get(key).cloned()
    }
}
