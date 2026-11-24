use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::history::data::coordinates::BranchId;

use super::{
    denial::RelationalMergeRequestNormalizationDenial,
    digest::normalized_merge_request_digest,
    raw::{MergeExecutionRequest, MergeIntent, MergePlanningRequest},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeScope {
    FullBranch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeRequestFamily {
    FullBranchReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeCorrespondencePosture {
    Advisory,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeSchemaReconciliationPosture {
    Participate,
    RequireCompatibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeTopologyIntent {
    PreserveTopologySemantics,
    RequireStrictTopologyStability,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizedRelationalMergeRequest {
    family: RelationalMergeRequestFamily,
    scope: RelationalMergeScope,
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
    correspondence_posture: RelationalMergeCorrespondencePosture,
    schema_reconciliation_posture: RelationalMergeSchemaReconciliationPosture,
    topology_intent: RelationalMergeTopologyIntent,
    request_digest: String,
}

impl PartialEq for NormalizedRelationalMergeRequest {
    fn eq(&self, other: &Self) -> bool {
        self.family == other.family
            && self.scope == other.scope
            && self.target_branch == other.target_branch
            && self.source_branch == other.source_branch
            && self.merge_intent == other.merge_intent
            && self.correspondence_posture == other.correspondence_posture
            && self.schema_reconciliation_posture == other.schema_reconciliation_posture
            && self.topology_intent == other.topology_intent
            && self.request_digest == other.request_digest
    }
}

impl Eq for NormalizedRelationalMergeRequest {}

impl NormalizedRelationalMergeRequest {
    pub fn admit_full_branch(
        target_branch: BranchId,
        source_branch: BranchId,
        merge_intent: MergeIntent,
        correspondence_posture: RelationalMergeCorrespondencePosture,
        schema_reconciliation_posture: RelationalMergeSchemaReconciliationPosture,
        topology_intent: RelationalMergeTopologyIntent,
    ) -> Result<Self, RelationalMergeRequestNormalizationDenial> {
        if correspondence_posture != RelationalMergeCorrespondencePosture::Advisory {
            return Err(
                RelationalMergeRequestNormalizationDenial::UnsupportedCorrespondencePosture {
                    posture: correspondence_posture,
                },
            );
        }
        if schema_reconciliation_posture != RelationalMergeSchemaReconciliationPosture::Participate
        {
            return Err(
                RelationalMergeRequestNormalizationDenial::UnsupportedSchemaReconciliationPosture {
                    posture: schema_reconciliation_posture,
                },
            );
        }
        if topology_intent != RelationalMergeTopologyIntent::PreserveTopologySemantics {
            return Err(
                RelationalMergeRequestNormalizationDenial::UnsupportedTopologyIntent {
                    intent: topology_intent,
                },
            );
        }

        let mut request = Self {
            family: RelationalMergeRequestFamily::FullBranchReconciliation,
            scope: RelationalMergeScope::FullBranch,
            target_branch,
            source_branch,
            merge_intent,
            correspondence_posture,
            schema_reconciliation_posture,
            topology_intent,
            request_digest: String::new(),
        };
        request.request_digest = normalized_merge_request_digest(&request);
        Ok(request)
    }

    pub fn from_planning_request(
        request: MergePlanningRequest,
    ) -> Result<Self, RelationalMergeRequestNormalizationDenial> {
        Self::admit_full_branch(
            request.target_branch().clone(),
            request.source_branch().clone(),
            request.merge_intent(),
            RelationalMergeCorrespondencePosture::Advisory,
            RelationalMergeSchemaReconciliationPosture::Participate,
            RelationalMergeTopologyIntent::PreserveTopologySemantics,
        )
    }

    pub fn from_execution_request(
        request: MergeExecutionRequest,
    ) -> Result<Self, RelationalMergeRequestNormalizationDenial> {
        Self::admit_full_branch(
            request.target_branch().clone(),
            request.source_branch().clone(),
            request.merge_intent(),
            RelationalMergeCorrespondencePosture::Advisory,
            RelationalMergeSchemaReconciliationPosture::Participate,
            RelationalMergeTopologyIntent::PreserveTopologySemantics,
        )
    }

    pub fn family(&self) -> RelationalMergeRequestFamily {
        self.family
    }

    pub fn scope(&self) -> RelationalMergeScope {
        self.scope
    }

    pub fn target_branch(&self) -> &BranchId {
        &self.target_branch
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    pub fn merge_intent(&self) -> MergeIntent {
        self.merge_intent
    }

    pub fn correspondence_posture(&self) -> RelationalMergeCorrespondencePosture {
        self.correspondence_posture
    }

    pub fn schema_reconciliation_posture(&self) -> RelationalMergeSchemaReconciliationPosture {
        self.schema_reconciliation_posture
    }

    pub fn topology_intent(&self) -> RelationalMergeTopologyIntent {
        self.topology_intent
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct NormalizedRelationalMergeRequestWire {
    family: RelationalMergeRequestFamily,
    scope: RelationalMergeScope,
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
    correspondence_posture: RelationalMergeCorrespondencePosture,
    schema_reconciliation_posture: RelationalMergeSchemaReconciliationPosture,
    topology_intent: RelationalMergeTopologyIntent,
    request_digest: String,
}

impl<'de> Deserialize<'de> for NormalizedRelationalMergeRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = NormalizedRelationalMergeRequestWire::deserialize(deserializer)?;
        if wire.family != RelationalMergeRequestFamily::FullBranchReconciliation {
            return Err(D::Error::custom(
                "normalized merge request family is not admitted by this milestone",
            ));
        }
        if wire.scope != RelationalMergeScope::FullBranch {
            return Err(D::Error::custom(
                "normalized merge request scope is not admitted by this milestone",
            ));
        }
        let admitted = NormalizedRelationalMergeRequest::admit_full_branch(
            wire.target_branch,
            wire.source_branch,
            wire.merge_intent,
            wire.correspondence_posture,
            wire.schema_reconciliation_posture,
            wire.topology_intent,
        )
        .map_err(|error| D::Error::custom(format!("{error:?}")))?;
        if admitted.request_digest != wire.request_digest {
            return Err(D::Error::custom(
                "normalized merge request digest does not match admitted request truth",
            ));
        }
        Ok(admitted)
    }
}

impl From<NormalizedRelationalMergeRequest> for MergeExecutionRequest {
    fn from(value: NormalizedRelationalMergeRequest) -> Self {
        Self::new(value.target_branch, value.source_branch, value.merge_intent)
    }
}
