//! Private durable evidence. Decoding never admits capability access.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedRole {
    version: u16,
    requirement: String,
    actor: ActorState,
    sources: Vec<u8>,
    source_version: u16,
    record: ActorState,
}
#[derive(Serialize, Deserialize)]
enum ActorState {
    NotYet,
    Present(EntityId),
}

impl WorkflowRecordedActorObservation {
    pub(in crate::domain_computation) fn validate_encoding(encoded: &str) -> Result<(), ()> {
        Self::parse_encoding(encoded).map(|_| ())
    }
    fn parse_encoding(encoded: &str) -> Result<RecordedRole, ()> {
        let value: RecordedRole = serde_json::from_str(encoded).map_err(|_| ())?;
        if value.version != 1
            || value.source_version != crate::domain_computation::primary_graph::application_checkpoint::facts::WIRE_VERSION
            || Requirement::from_persistence_identity(&value.requirement).is_none()
            || value.sources.len() < 5
            || matches!(value.actor, ActorState::Present(_)) && matches!(value.record, ActorState::NotYet) {
            return Err(());
        }
        Ok(value)
    }
    pub(in crate::domain_computation) fn encode(&self) -> Result<String, ()> {
        serde_json::to_string(&RecordedRole {
            version: 1, requirement: self.requirement.persistence_identity().to_owned(),
            actor: self.actor.map_or(ActorState::NotYet, ActorState::Present),
            sources: self.dependencies.wire().to_vec(),
            source_version: crate::domain_computation::primary_graph::application_checkpoint::facts::WIRE_VERSION,
            record: self.record.map_or(ActorState::NotYet, ActorState::Present),
        }).map_err(|_| ())
    }
    pub(in crate::domain_computation) fn decode(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        encoded: &str,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let invalid = || {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
                "workflow recorded role wire is invalid",
            )
        };
        let value = Self::parse_encoding(encoded).map_err(|_| invalid())?;
        let requirement =
            Requirement::from_persistence_identity(&value.requirement).ok_or_else(invalid)?;
        let dependencies = WorkflowActorDependencies::restore(execution, &value.sources)?;
        Ok(Self {
            requirement,
            dependencies,
            record: match value.record {
                ActorState::NotYet => None,
                ActorState::Present(record) => Some(record),
            },
            actor: match value.actor {
                ActorState::NotYet => None,
                ActorState::Present(actor) => Some(actor),
            },
        })
    }
    pub(in crate::domain_computation) fn combined_identity(
        &self,
        relational: &[u8; 32],
    ) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.query.native-recorded-role.v1");
        digest.update(relational);
        let requirement = self.requirement.persistence_identity();
        digest.update((requirement.len() as u64).to_le_bytes());
        digest.update(requirement.as_bytes());
        match self.actor {
            None => digest.update([0]),
            Some(actor) => {
                digest.update([1]);
                digest.update(actor.partition_value().to_le_bytes());
                digest.update(actor.local_slot_value().to_le_bytes());
                digest.update(actor.generation_value().to_le_bytes());
            }
        }
        digest.update([u8::from(self.record.is_some())]);
        if let Some(record) = self.record {
            digest.update(record.partition_value().to_le_bytes());
            digest.update(record.local_slot_value().to_le_bytes());
            digest.update(record.generation_value().to_le_bytes());
        }
        digest.update((self.dependencies.wire().len() as u64).to_le_bytes());
        digest.update(self.dependencies.wire());
        digest.finalize().into()
    }
    pub(in crate::domain_computation) fn same_original_source(&self, other: &Self) -> bool {
        self.requirement == other.requirement
            && self.actor == other.actor
            && self.record == other.record
            && self.dependencies.wire() == other.dependencies.wire()
    }
    pub(in crate::domain_computation) fn is_present(&self) -> bool {
        self.actor.is_some()
    }
}
