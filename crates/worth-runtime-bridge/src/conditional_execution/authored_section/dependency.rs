//! Portable semantic claims omit source installation and runtime authority.
use super::BridgeAuthoredSectionDenial as D;
use crate::correspondence::{BridgeSemanticDependencyCandidate, BridgeSemanticLocality};
use crate::relational_source::identity_parts::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{
    AspectBinding, AuthoritativeAspectChangeKind, PortableAspectContract,
};
mod memory;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Meaning {
    pub(super) stage: Option<String>,
    pub(super) node: String,
    pub(super) ordinal: u32,
    pub(super) graph_role: String,
    pub(super) source_record: Option<Record>,
    pub(super) observation_record: Option<Record>,
    pub(super) contract: PortableAspectContract,
    pub(super) projection_paths: Vec<Vec<String>>,
    pub(super) binding: AspectBinding,
    pub(super) locality: BridgeSemanticLocality,
    pub(super) changes: Vec<AuthoritativeAspectChangeKind>,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    kind: RecordKind,
    partition: u32,
    slot: u64,
    generation: u32,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
enum RecordKind {
    Entity,
    Relation,
}
impl Record {
    fn capture(record: RelationalBridgeRecordIdentityParts) -> Self {
        Self {
            kind: match record.kind() {
                RelationalBridgeRecordIdentityKind::Entity => RecordKind::Entity,
                RelationalBridgeRecordIdentityKind::Relation => RecordKind::Relation,
            },
            partition: record.partition_id(),
            slot: record.local_slot(),
            generation: record.generation(),
        }
    }
    fn parts(self) -> RelationalBridgeRecordIdentityParts {
        RelationalBridgeRecordIdentityParts::new(
            match self.kind {
                RecordKind::Entity => RelationalBridgeRecordIdentityKind::Entity,
                RecordKind::Relation => RelationalBridgeRecordIdentityKind::Relation,
            },
            self.partition,
            self.slot,
            self.generation,
        )
    }
}
#[derive(Debug)]
pub(super) struct FundedMeaning {
    pub(super) data: Meaning,
    _memory: Option<ExecutionMemoryReservation>,
}

/// Recorded meaning is descriptive. A fresh source owner must supply its binding.
#[derive(Clone, Debug)]
pub struct BridgeAuthoredDependencyDescription {
    meaning: Arc<FundedMeaning>,
}
impl BridgeAuthoredDependencyDescription {
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        candidate: &BridgeSemanticDependencyCandidate,
    ) -> Result<Self, D> {
        let charge = memory::capture(candidate)?;
        let funding = execution
            .reserve_retained_memory(charge)
            .map_err(D::Request)?;
        let data = Meaning {
            stage: candidate.source_stage_identity().map(str::to_owned),
            node: candidate.source_node_identity().to_owned(),
            ordinal: candidate
                .dependency_ordinal()
                .try_into()
                .map_err(|_| D::SizeLimit)?,
            graph_role: candidate.declared_graph_role().to_owned(),
            source_record: candidate.source_record_identity().map(Record::capture),
            observation_record: candidate.observation_record_identity().map(Record::capture),
            contract: PortableAspectContract::from_contract(candidate.contract()),
            projection_paths: candidate
                .projection_mask()
                .paths()
                .iter()
                .map(|path| {
                    path.fields()
                        .iter()
                        .map(|field| field.as_str().to_owned())
                        .collect()
                })
                .collect(),
            binding: candidate.binding().clone(),
            locality: candidate.locality().clone(),
            changes: candidate.relevant_changes().to_vec(),
        };
        Ok(Self {
            meaning: Arc::new(FundedMeaning {
                data,
                _memory: funding,
            }),
        })
    }
    pub(super) fn data(&self) -> &Meaning {
        &self.meaning.data
    }
    pub(super) fn decode(execution: &ExecutionRequest<'_, '_>, bytes: &[u8]) -> Result<Self, D> {
        let _scratch = execution
            .reserve_retained_memory((bytes.len() as u64).checked_mul(2).ok_or(D::SizeLimit)?)
            .map_err(D::Request)?;
        let quote = memory::decode(bytes)?;
        let preparation = execution
            .reserve_retained_memory(quote)
            .map_err(D::Request)?;
        let data: Meaning = serde_json::from_slice(bytes).map_err(|_| D::Malformed)?;
        if data.node.trim().is_empty()
            || data.graph_role.trim().is_empty()
            || data.changes.is_empty()
        {
            return Err(D::Malformed);
        }
        execution.consult().map_err(D::Request)?;
        let funding = execution
            .reserve_retained_memory(memory::retained(&data, bytes.len())?)
            .map_err(D::Request)?;
        drop(preparation);
        Ok(Self {
            meaning: Arc::new(FundedMeaning {
                data,
                _memory: funding,
            }),
        })
    }
    pub fn node_identity(&self) -> &str {
        &self.meaning.data.node
    }
    pub fn stage_identity(&self) -> Option<&str> {
        self.meaning.data.stage.as_deref()
    }
    pub fn dependency_ordinal(&self) -> usize {
        self.meaning.data.ordinal as usize
    }
    pub fn graph_role(&self) -> &str {
        &self.meaning.data.graph_role
    }
    pub fn contract(&self) -> &PortableAspectContract {
        &self.meaning.data.contract
    }
    pub fn projection_paths(&self) -> &[Vec<String>] {
        &self.meaning.data.projection_paths
    }
    pub fn binding(&self) -> &AspectBinding {
        &self.meaning.data.binding
    }
    pub fn locality(&self) -> &BridgeSemanticLocality {
        &self.meaning.data.locality
    }
    pub fn relevant_changes(&self) -> &[AuthoritativeAspectChangeKind] {
        &self.meaning.data.changes
    }
    pub fn source_record(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        self.meaning.data.source_record.map(Record::parts)
    }
    pub fn observation_record(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        self.meaning.data.observation_record.map(Record::parts)
    }
    pub(in crate::conditional_execution) fn admits_fresh_candidate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        fresh: &BridgeSemanticDependencyCandidate,
    ) -> Result<bool, D> {
        Ok(self.meaning.data == Self::capture(execution, fresh)?.meaning.data)
    }
}
