use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::merge::data::{
    DeletionExecutionClass, LoweredMergeBlockedReason, MergeExecutionReadiness,
    MergeResolutionClass, TopologyExecutionClass,
};
use crate::merge::{
    blocked_reason_for_deletion_class, blocked_reason_for_topology_resolution_class,
};
use crate::transactions::data::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeTopologyStrategyWitnessRow {
    record: RecordRef,
    target_record: Option<RecordRef>,
    topology_class: TopologyExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    row_digest: String,
}

impl RelationalMergeTopologyStrategyWitnessRow {
    pub(crate) fn retained(
        record: RecordRef,
        target_record: Option<RecordRef>,
        topology_class: TopologyExecutionClass,
        readiness: MergeExecutionReadiness,
        blocked_reason: Option<LoweredMergeBlockedReason>,
    ) -> Self {
        let row_digest = topology_row_digest(
            &record,
            target_record.as_ref(),
            topology_class,
            readiness,
            blocked_reason,
        );
        Self {
            record,
            target_record,
            topology_class,
            readiness,
            blocked_reason,
            row_digest,
        }
    }

    pub fn record(&self) -> &RecordRef {
        &self.record
    }
    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }
    pub fn topology_class(&self) -> TopologyExecutionClass {
        self.topology_class
    }
    pub fn readiness(&self) -> MergeExecutionReadiness {
        self.readiness
    }
    pub fn blocked_reason(&self) -> Option<LoweredMergeBlockedReason> {
        self.blocked_reason
    }
    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }

    pub(crate) fn retains_honest_truth(&self) -> bool {
        topology_truth_matches(self.topology_class, self.readiness, self.blocked_reason)
            && self.row_digest
                == topology_row_digest(
                    &self.record,
                    self.target_record.as_ref(),
                    self.topology_class,
                    self.readiness,
                    self.blocked_reason,
                )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationalMergeDeletionStrategyWitnessRow {
    record: RecordRef,
    target_record: Option<RecordRef>,
    deletion_class: DeletionExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    row_digest: String,
}

impl RelationalMergeDeletionStrategyWitnessRow {
    pub(crate) fn retained(
        record: RecordRef,
        target_record: Option<RecordRef>,
        deletion_class: DeletionExecutionClass,
        readiness: MergeExecutionReadiness,
        blocked_reason: Option<LoweredMergeBlockedReason>,
    ) -> Self {
        let row_digest = deletion_row_digest(
            &record,
            target_record.as_ref(),
            deletion_class,
            readiness,
            blocked_reason,
        );
        Self {
            record,
            target_record,
            deletion_class,
            readiness,
            blocked_reason,
            row_digest,
        }
    }

    pub fn record(&self) -> &RecordRef {
        &self.record
    }
    pub fn target_record(&self) -> Option<&RecordRef> {
        self.target_record.as_ref()
    }
    pub fn deletion_class(&self) -> DeletionExecutionClass {
        self.deletion_class
    }
    pub fn readiness(&self) -> MergeExecutionReadiness {
        self.readiness
    }
    pub fn blocked_reason(&self) -> Option<LoweredMergeBlockedReason> {
        self.blocked_reason
    }
    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }

    pub(crate) fn retains_honest_truth(&self) -> bool {
        deletion_truth_matches(self.deletion_class, self.readiness, self.blocked_reason)
            && self.row_digest
                == deletion_row_digest(
                    &self.record,
                    self.target_record.as_ref(),
                    self.deletion_class,
                    self.readiness,
                    self.blocked_reason,
                )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeTopologyStrategyWitnessRowWire {
    record: RecordRef,
    target_record: Option<RecordRef>,
    topology_class: TopologyExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    row_digest: String,
}

impl<'de> Deserialize<'de> for RelationalMergeTopologyStrategyWitnessRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeTopologyStrategyWitnessRowWire::deserialize(deserializer)?;
        if !topology_truth_matches(wire.topology_class, wire.readiness, wire.blocked_reason) {
            return Err(D::Error::custom(
                "merge strategy topology row truth does not match retained topology posture",
            ));
        }
        let row_digest = topology_row_digest(
            &wire.record,
            wire.target_record.as_ref(),
            wire.topology_class,
            wire.readiness,
            wire.blocked_reason,
        );
        if row_digest != wire.row_digest {
            return Err(D::Error::custom(
                "merge strategy topology row digest does not match retained truth",
            ));
        }
        Ok(Self {
            record: wire.record,
            target_record: wire.target_record,
            topology_class: wire.topology_class,
            readiness: wire.readiness,
            blocked_reason: wire.blocked_reason,
            row_digest: wire.row_digest,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RelationalMergeDeletionStrategyWitnessRowWire {
    record: RecordRef,
    target_record: Option<RecordRef>,
    deletion_class: DeletionExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
    row_digest: String,
}

impl<'de> Deserialize<'de> for RelationalMergeDeletionStrategyWitnessRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RelationalMergeDeletionStrategyWitnessRowWire::deserialize(deserializer)?;
        if !deletion_truth_matches(wire.deletion_class, wire.readiness, wire.blocked_reason) {
            return Err(D::Error::custom(
                "merge strategy deletion row truth does not match retained deletion posture",
            ));
        }
        let row_digest = deletion_row_digest(
            &wire.record,
            wire.target_record.as_ref(),
            wire.deletion_class,
            wire.readiness,
            wire.blocked_reason,
        );
        if row_digest != wire.row_digest {
            return Err(D::Error::custom(
                "merge strategy deletion row digest does not match retained truth",
            ));
        }
        Ok(Self {
            record: wire.record,
            target_record: wire.target_record,
            deletion_class: wire.deletion_class,
            readiness: wire.readiness,
            blocked_reason: wire.blocked_reason,
            row_digest: wire.row_digest,
        })
    }
}

fn topology_truth_matches(
    topology_class: TopologyExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
) -> bool {
    match readiness {
        MergeExecutionReadiness::Admitted => {
            topology_class == TopologyExecutionClass::RelationEndpointStable
                && blocked_reason.is_none()
        }
        MergeExecutionReadiness::Blocked => {
            blocked_reason
                == Some(blocked_reason_for_topology_resolution_class(
                    MergeResolutionClass::Topology(topology_class),
                ))
        }
        MergeExecutionReadiness::Rejected => false,
    }
}

fn deletion_truth_matches(
    deletion_class: DeletionExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
) -> bool {
    match readiness {
        MergeExecutionReadiness::Admitted => {
            deletion_class == DeletionExecutionClass::DeletedOnBothSides && blocked_reason.is_none()
        }
        MergeExecutionReadiness::Blocked => {
            blocked_reason
                == Some(blocked_reason_for_deletion_class(match deletion_class {
                    DeletionExecutionClass::SourceDeletedTargetLive => {
                        crate::merge::data::DeletionMergeClass::SourceDeletedTargetLive
                    }
                    DeletionExecutionClass::SourceLiveTargetDeleted => {
                        crate::merge::data::DeletionMergeClass::SourceLiveTargetDeleted
                    }
                    DeletionExecutionClass::DeletedOnBothSides => {
                        crate::merge::data::DeletionMergeClass::DeletedOnBothSides
                    }
                    DeletionExecutionClass::DeletedVsModified => {
                        crate::merge::data::DeletionMergeClass::DeletedVsModified
                    }
                    DeletionExecutionClass::DeletedVsRewired => {
                        crate::merge::data::DeletionMergeClass::DeletedVsRewired
                    }
                }))
        }
        MergeExecutionReadiness::Rejected => false,
    }
}

fn topology_row_digest(
    record: &RecordRef,
    target_record: Option<&RecordRef>,
    topology_class: TopologyExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
) -> String {
    let digest = Sha256::digest(
        rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.topology_row.v1",
            record,
            target_record,
            topology_class,
            readiness,
            blocked_reason,
        ))
        .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "strategy witness topology row must encode"),
    );
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn deletion_row_digest(
    record: &RecordRef,
    target_record: Option<&RecordRef>,
    deletion_class: DeletionExecutionClass,
    readiness: MergeExecutionReadiness,
    blocked_reason: Option<LoweredMergeBlockedReason>,
) -> String {
    let digest = Sha256::digest(
        rmp_serde::to_vec_named(&(
            "worth.relational.merge.strategy_witness.deletion_row.v1",
            record,
            target_record,
            deletion_class,
            readiness,
            blocked_reason,
        ))
        .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "strategy witness deletion row must encode"),
    );
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
