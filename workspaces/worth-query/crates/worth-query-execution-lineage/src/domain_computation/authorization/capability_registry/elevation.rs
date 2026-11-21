use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_relational::facade::identity::KindId;
use worth_runtime_bridge::facade::BridgeAuthorizationCorrespondenceIdentity;

pub struct WorthQueryCapabilityUpperBoundBindings {
    pub correspondence:
        BridgeAuthorizationCorrespondenceIdentity,
    pub path_count: usize,
    pub rules:
        Vec<super::WorthQueryCapabilityRuleBinding>,
    pub decision_rules:
        super::WorthQueryCapabilityDecisionRuleBindings,
}

pub struct WorthQueryCapabilityElevationBindings {
    pub elevation_kind: KindId,
    pub active_path_index: usize,
    pub expired_path_index: usize,
    pub self_approval_path_index: usize,
    pub temporal:
        WorthQueryCapabilityElevationTemporalBindings,
    pub approver_conflict_requirements:
        Vec<Vec<usize>>,
    pub lifecycle:
        WorthQueryCapabilityElevationLifecycleBindings,
}

pub struct WorthQueryCapabilityElevationLifecycleBindings
{
    pub review_kind: KindId,
    pub identity:
        worth_foundational::facade::AspectFieldLocator,
    pub reason:
        worth_foundational::facade::AspectFieldLocator,
    pub status:
        worth_foundational::facade::AspectFieldLocator,
    pub closed_at:
        worth_foundational::facade::AspectFieldLocator,
    pub review_identity:
        worth_foundational::facade::AspectFieldLocator,
    pub review_type:
        worth_foundational::facade::AspectFieldLocator,
    pub review_type_value:
        worth_foundational::facade::AspectValue,
    pub review_status:
        worth_foundational::facade::AspectFieldLocator,
    pub reviewed_at:
        worth_foundational::facade::AspectFieldLocator,
    pub requester_relation: KindId,
    pub approver_relation: KindId,
    pub grant_relation: KindId,
    pub resource_relation: Option<KindId>,
    pub review_relation: KindId,
    pub review_scope_relation: KindId,
    pub reviewer_relation: KindId,
    pub requested:
        worth_foundational::facade::AspectValue,
    pub approved:
        worth_foundational::facade::AspectValue,
    pub expired:
        worth_foundational::facade::AspectValue,
    pub revoked:
        worth_foundational::facade::AspectValue,
    pub review_required:
        worth_foundational::facade::AspectValue,
    pub review_completed:
        worth_foundational::facade::AspectValue,
    pub maximum_duration: std::time::Duration,
}

pub struct WorthQueryCapabilityElevationTemporalBindings
{
    pub timeline:
        ApplicationCapabilityValidityTimeline,
    pub not_before_path_index: usize,
    pub not_after_path_index: usize,
    pub not_before:
        worth_foundational::facade::AspectFieldLocator,
    pub not_after:
        worth_foundational::facade::AspectFieldLocator,
}

impl WorthQueryCapabilityElevationBindings {
    pub fn new(
        elevation_kind: KindId,
        active_path_index: usize,
        expired_path_index: usize,
        self_approval_path_index: usize,
        temporal: WorthQueryCapabilityElevationTemporalBindings,
        approver_conflict_requirements: Vec<Vec<usize>>,
        lifecycle: WorthQueryCapabilityElevationLifecycleBindings,
    ) -> Self {
        Self {
            elevation_kind,
            active_path_index,
            expired_path_index,
            self_approval_path_index,
            temporal,
            approver_conflict_requirements,
            lifecycle,
        }
    }
}

impl WorthQueryCapabilityElevationTemporalBindings {
    pub fn new(
        timeline: ApplicationCapabilityValidityTimeline,
        not_before_path_index: usize,
        not_after_path_index: usize,
        not_before: worth_foundational::facade::AspectFieldLocator,
        not_after: worth_foundational::facade::AspectFieldLocator,
    ) -> Self {
        Self {
            timeline,
            not_before_path_index,
            not_after_path_index,
            not_before,
            not_after,
        }
    }
}
