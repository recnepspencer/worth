//! Query-owned dispatch outbox record co-committed with the mutation (R8.25).

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalDigestId,
    InternedString,
};
use worth_query_installation::facade::{
    InstalledExternalEffectContract, WorthQueryExternalEffectCorrelationFamily,
};
use worth_relational::facade::transactions::{
    CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent,
};

use super::correlation::ExternalEffectCorrelationIdentity;

/// Durable outbox fields bound into the operation's MutationIntent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDispatchOutboxRecord {
    correlation: ExternalEffectCorrelationIdentity,
    correlation_family: WorthQueryExternalEffectCorrelationFamily,
    effect: String,
    protocol_identity: BoundaryProtocolIdentity,
    protocol_version: BoundaryProtocolVersion,
    maximum_payload_bytes: u64,
    payload: Vec<u8>,
    outcome_identity: u64,
}

/// Fully decoded durable fields returned by the Relational owner read.
pub(crate) struct WorthQueryDispatchOutboxRestoredFields {
    pub correlation: ExternalEffectCorrelationIdentity,
    pub correlation_family: String,
    pub effect: String,
    pub protocol_identity: BoundaryProtocolIdentity,
    pub protocol_version: BoundaryProtocolVersion,
    pub maximum_payload_bytes: u64,
    pub payload: Vec<u8>,
    pub outcome_identity: u64,
}

/// The Query-owned outbox record and the exact create reference submitted with it.
///
/// Keeping these together prevents the post-commit lane from reconstructing record identity
/// from persisted field content.
pub(crate) struct WorthQueryPendingDispatchOutbox {
    record: WorthQueryDispatchOutboxRecord,
    created_entity: CreatedEntityRef,
}

impl WorthQueryPendingDispatchOutbox {
    pub(crate) const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        &self.record
    }

    pub(crate) const fn created_entity(&self) -> &CreatedEntityRef {
        &self.created_entity
    }
}

impl WorthQueryDispatchOutboxRecord {
    /// Derive one durable record from the installed contract as a whole.
    ///
    /// Contract metadata is deliberately not accepted as independent
    /// parameters: the producer cannot combine the family, effect, wire
    /// identity, or byte bound from different declarations.
    pub(crate) fn from_installed_contract(
        correlation: ExternalEffectCorrelationIdentity,
        contract: &InstalledExternalEffectContract,
        payload: Vec<u8>,
        outcome_identity: u64,
    ) -> Option<Self> {
        let InstalledExternalEffectContract::Declared {
            correlation_family,
            effect,
            protocol,
            maximum_payload_bytes,
            ..
        } = contract
        else {
            return None;
        };
        Some(Self {
            correlation,
            correlation_family: correlation_family.clone(),
            effect: effect.clone(),
            protocol_identity: protocol.identity().clone(),
            protocol_version: protocol.version(),
            maximum_payload_bytes: *maximum_payload_bytes,
            payload,
            outcome_identity,
        })
    }

    pub const fn correlation(&self) -> &ExternalEffectCorrelationIdentity {
        &self.correlation
    }

    pub const fn correlation_family(&self) -> &WorthQueryExternalEffectCorrelationFamily {
        &self.correlation_family
    }

    pub fn effect(&self) -> &str {
        &self.effect
    }

    pub const fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        &self.protocol_identity
    }

    pub const fn protocol_version(&self) -> BoundaryProtocolVersion {
        self.protocol_version
    }

    pub const fn maximum_payload_bytes(&self) -> u64 {
        self.maximum_payload_bytes
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub const fn outcome_identity(&self) -> u64 {
        self.outcome_identity
    }

    pub(crate) fn restore(fields: WorthQueryDispatchOutboxRestoredFields) -> Option<Self> {
        Some(Self {
            correlation: fields.correlation,
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new(
                fields.correlation_family,
            )
            .ok()?,
            effect: fields.effect,
            protocol_identity: fields.protocol_identity,
            protocol_version: fields.protocol_version,
            maximum_payload_bytes: fields.maximum_payload_bytes,
            payload: fields.payload,
            outcome_identity: fields.outcome_identity,
        })
    }
}

/// Layout locators required to create a dispatch-outbox entity.
#[derive(Clone, Debug)]
pub struct WorthQueryDispatchOutboxLayout {
    pub entity_kind: worth_relational::facade::identity::KindId,
    pub correlation_locator: worth_foundational::facade::AspectFieldLocator,
    pub family_locator: worth_foundational::facade::AspectFieldLocator,
    pub effect_locator: worth_foundational::facade::AspectFieldLocator,
    pub protocol_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub protocol_version_locator: worth_foundational::facade::AspectFieldLocator,
    pub maximum_payload_bytes_locator: worth_foundational::facade::AspectFieldLocator,
    pub payload_locator: worth_foundational::facade::AspectFieldLocator,
    pub outcome_identity_locator: worth_foundational::facade::AspectFieldLocator,
}

/// Build a create intent for a declared external effect. Returns `None` when
/// the operation declares no external effect (R8.4 — pay exactly zero).
#[cfg(test)]
pub(crate) fn dispatch_outbox_create_intent(
    layout: Option<&WorthQueryDispatchOutboxLayout>,
    record: Option<&WorthQueryDispatchOutboxRecord>,
) -> Option<MutationIntent> {
    bind_dispatch_outbox_create_intent(
        layout,
        record,
        worth_relational::facade::identity::PartitionId::main(),
    )
    .map(|(intent, _)| intent)
}

pub(crate) fn bind_dispatch_outbox_create_intent(
    layout: Option<&WorthQueryDispatchOutboxLayout>,
    record: Option<&WorthQueryDispatchOutboxRecord>,
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> Option<(MutationIntent, WorthQueryPendingDispatchOutbox)> {
    let (layout, record) = match (layout, record) {
        (Some(layout), Some(record)) => (layout, record),
        _ => return None,
    };
    let correlation_hex = hex_digest(record.correlation().digest());
    let fields = BTreeMap::from([
        (
            layout.correlation_locator.clone(),
            AspectValue::String(InternedString::from(correlation_hex.clone())),
        ),
        (
            layout.family_locator.clone(),
            AspectValue::String(InternedString::from(
                record.correlation_family.as_str().to_owned(),
            )),
        ),
        (
            layout.effect_locator.clone(),
            AspectValue::String(InternedString::from(record.effect.clone())),
        ),
        (
            layout.protocol_identity_locator.clone(),
            AspectValue::String(InternedString::from(
                record.protocol_identity.as_str().to_owned(),
            )),
        ),
        (
            layout.protocol_version_locator.clone(),
            AspectValue::UInt64(u64::from(record.protocol_version.get())),
        ),
        (
            layout.maximum_payload_bytes_locator.clone(),
            AspectValue::UInt64(record.maximum_payload_bytes),
        ),
        (
            layout.payload_locator.clone(),
            AspectValue::String(InternedString::from(hex_bytes(&record.payload))),
        ),
        (
            layout.outcome_identity_locator.clone(),
            AspectValue::UInt64(record.outcome_identity),
        ),
    ]);
    let created_entity = CreatedEntityRef {
        partition_id: mutation_partition,
        kind_id: layout.entity_kind,
        client_key: worth_relational::facade::symbols::ClientKey::raw(format!(
            "worth-query-dispatch-outbox:{correlation_hex}"
        )),
    };
    let intent = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: created_entity.partition_id,
        kind_id: created_entity.kind_id,
        client_key: created_entity.client_key.clone(),
        fields: worth_relational::facade::transactions::AspectFieldPatch::from(fields),
    }));
    Some((
        intent,
        WorthQueryPendingDispatchOutbox {
            record: record.clone(),
            created_entity,
        },
    ))
}

fn hex_digest(digest: &CanonicalDigestId) -> String {
    hex_bytes(digest.bytes())
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
