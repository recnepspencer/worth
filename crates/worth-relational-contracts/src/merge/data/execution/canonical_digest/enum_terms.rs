use crate::merge::data::causal::MergeRecordCausalDisposition;
use crate::merge::data::conflicts::MergeConflictClass;
use crate::merge::data::execution::executable_plan::{
    DeletedOnBothSidesSemantics, MergeLineageContinuityVerdict,
};
use crate::merge::data::policy::{
    AspectMergePolicyKind, MergeManualResolutionClass, MergePolicyDecisionBoundary,
    MergePolicyOwnershipSurface, MergePolicyProofBoundary, MergePolicyRejectClass,
    MergeResolutionClass, TopologyExecutionClass,
};

impl super::CanonicalDigestBytes {
    pub(super) fn merge_intent(&mut self, value: crate::merge::data::requests::raw::MergeIntent) {
        match value {
            crate::merge::data::requests::raw::MergeIntent::ReconcileIntoTarget => self.tag(1),
        }
    }

    pub(super) fn deleted_on_both_sides_semantics(&mut self, value: DeletedOnBothSidesSemantics) {
        match value {
            DeletedOnBothSidesSemantics::AuthoritativeMutualDeletionConvergence => self.tag(1),
        }
    }

    pub(super) fn merge_lineage_continuity(&mut self, value: MergeLineageContinuityVerdict) {
        match value {
            MergeLineageContinuityVerdict::Unchanged => self.tag(1),
            MergeLineageContinuityVerdict::Preserved => self.tag(2),
            MergeLineageContinuityVerdict::Transformed => self.tag(3),
        }
    }

    pub(super) fn merge_record_causal_disposition(&mut self, value: MergeRecordCausalDisposition) {
        match value {
            MergeRecordCausalDisposition::SourceOnly => self.tag(1),
            MergeRecordCausalDisposition::TargetOnly => self.tag(2),
            MergeRecordCausalDisposition::Equal => self.tag(3),
            MergeRecordCausalDisposition::SourceBeforeTarget => self.tag(4),
            MergeRecordCausalDisposition::SourceAfterTarget => self.tag(5),
            MergeRecordCausalDisposition::Concurrent => self.tag(6),
        }
    }

    pub(super) fn merge_conflict_class(&mut self, value: MergeConflictClass) {
        match value {
            MergeConflictClass::ExactSharedTruth => self.tag(1),
            MergeConflictClass::SourceOnlyAddition => self.tag(2),
            MergeConflictClass::SchemaDeclaredCorrespondence => self.tag(3),
            MergeConflictClass::Deletion(class) => {
                self.tag(4);
                self.tag(class as u8 + 1);
            }
            MergeConflictClass::DivergentVisibleState => self.tag(5),
            MergeConflictClass::StrategyIntentConflict => self.tag(6),
            MergeConflictClass::RelationEndpointDivergence => self.tag(7),
        }
    }

    pub(super) fn merge_resolution_class(&mut self, value: MergeResolutionClass) {
        match value {
            MergeResolutionClass::SourceOnlyAddition => self.tag(1),
            MergeResolutionClass::ExactSharedTruth => self.tag(2),
            MergeResolutionClass::SchemaDeclaredCorrespondence => self.tag(3),
            MergeResolutionClass::Deletion(class) => {
                self.tag(4);
                self.tag(class as u8 + 1);
            }
            MergeResolutionClass::Topology(class) => {
                self.tag(5);
                self.topology_execution_class(class);
            }
            MergeResolutionClass::DivergentVisibleState => self.tag(6),
        }
    }

    pub(super) fn merge_executable_class(
        &mut self,
        value: crate::merge::data::policy::MergeExecutableClass,
    ) {
        match value {
            crate::merge::data::policy::MergeExecutableClass::AdoptSourceRecord => self.tag(1),
            crate::merge::data::policy::MergeExecutableClass::PreserveSharedRecord => self.tag(2),
            crate::merge::data::policy::MergeExecutableClass::ReconcileRecord => self.tag(3),
            crate::merge::data::policy::MergeExecutableClass::ConvergeDeletedOnBothSides => {
                self.tag(4)
            }
        }
    }

    pub(super) fn merge_policy_proof_boundary(&mut self, value: MergePolicyProofBoundary) {
        self.merge_policy_ownership_surface(value.ownership_surface);
        self.merge_policy_decision_boundary(value.decision_boundary);
    }

    pub(super) fn aspect_merge_policy_kind(&mut self, value: &AspectMergePolicyKind) {
        match value {
            AspectMergePolicyKind::FailOnConflict => self.tag(1),
            AspectMergePolicyKind::LastWriterWins => self.tag(2),
            AspectMergePolicyKind::MonotonicCounter => self.tag(3),
            AspectMergePolicyKind::AdditiveSet => self.tag(4),
            AspectMergePolicyKind::PreferRicher => self.tag(5),
            AspectMergePolicyKind::Custom(custom) => {
                self.tag(6);
                self.str(&custom.name);
                self.u32(custom.semantic_version);
            }
        }
    }

    fn topology_execution_class(&mut self, value: TopologyExecutionClass) {
        match value {
            TopologyExecutionClass::RelationEndpointStable => self.tag(1),
            TopologyExecutionClass::RelationEndpointRewiredLocal => self.tag(2),
            TopologyExecutionClass::RelationEndpointRewiredEscalated => self.tag(3),
            TopologyExecutionClass::TopologyRegionConflict => self.tag(4),
        }
    }

    fn merge_policy_ownership_surface(&mut self, value: MergePolicyOwnershipSurface) {
        match value {
            MergePolicyOwnershipSurface::RuntimeOnly => self.tag(1),
            MergePolicyOwnershipSurface::ContainsCustomPolicy => self.tag(2),
        }
    }

    fn merge_policy_decision_boundary(&mut self, value: MergePolicyDecisionBoundary) {
        match value {
            MergePolicyDecisionBoundary::AutoResolved => self.tag(1),
            MergePolicyDecisionBoundary::RequiresManualResolution { class } => {
                self.tag(2);
                self.merge_manual_resolution_class(class);
            }
            MergePolicyDecisionBoundary::Reject { class } => {
                self.tag(3);
                self.merge_policy_reject_class(class);
            }
        }
    }

    fn merge_manual_resolution_class(&mut self, value: MergeManualResolutionClass) {
        match value {
            MergeManualResolutionClass::GenericRuntimeConflict => self.tag(1),
            MergeManualResolutionClass::StrategyIntentConflict => self.tag(2),
            MergeManualResolutionClass::MissingVisibleState => self.tag(3),
            MergeManualResolutionClass::MissingAncestorValueBasis => self.tag(4),
            MergeManualResolutionClass::UnvalidatedSchemaCorrespondence => self.tag(5),
            MergeManualResolutionClass::MixedAspectManualResolution => self.tag(6),
        }
    }

    fn merge_policy_reject_class(&mut self, value: MergePolicyRejectClass) {
        match value {
            MergePolicyRejectClass::BuiltInFailOnConflict => self.tag(1),
            MergePolicyRejectClass::LastWriterWinsCausalConflict => self.tag(2),
            MergePolicyRejectClass::InvalidBuiltInPolicyValueShape => self.tag(3),
            MergePolicyRejectClass::CustomPolicyRejected => self.tag(4),
            MergePolicyRejectClass::MixedAspectRejectClasses => self.tag(5),
        }
    }
}
