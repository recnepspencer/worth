use crate::merge::data::execution::executable_plan::{
    AdoptSourceRecordPlan, BoundExecutableMergeRecordPlan, ConvergeDeletedOnBothSidesRecordPlan,
    ExecutableAspectPlan, MergeExecutableRecordProvenance, PreserveSharedRecordPlan,
    ReconcileRecordPlan, ReconciledIdentityBasis, SharedTruthWitness, VisibleMergeRecordSnapshot,
};
use crate::merge::data::policy::ResolvedAspectMergePolicy;
use crate::merge::data::visible_record::VisibleMergeRecordKind;

impl super::CanonicalDigestBytes {
    pub(super) fn executable_record_plans(&mut self, values: &[BoundExecutableMergeRecordPlan]) {
        self.usize(values.len());
        for value in values {
            self.executable_record_plan(value);
        }
    }

    fn executable_record_plan(&mut self, value: &BoundExecutableMergeRecordPlan) {
        match value {
            BoundExecutableMergeRecordPlan::AdoptSource(plan) => self.adopt_source_plan(plan),
            BoundExecutableMergeRecordPlan::PreserveShared(plan) => self.preserve_shared_plan(plan),
            BoundExecutableMergeRecordPlan::Reconcile(plan) => self.reconcile_plan(plan),
            BoundExecutableMergeRecordPlan::ConvergeDeletedOnBothSides(plan) => {
                self.deleted_on_both_sides_plan(plan)
            }
        }
    }

    fn adopt_source_plan(&mut self, plan: &AdoptSourceRecordPlan) {
        self.tag(1);
        self.record_ref(&plan.source_record);
        self.visible_record_kind(plan.record_kind.clone());
        self.visible_snapshot(&plan.source_visible_snapshot);
        self.provenance(&plan.provenance);
        self.executable_aspect_plans(&plan.aspect_plan);
    }

    fn preserve_shared_plan(&mut self, plan: &PreserveSharedRecordPlan) {
        self.tag(2);
        self.record_ref(&plan.record);
        self.optional_record_ref(plan.target_record.as_ref());
        self.shared_truth_witness(&plan.equality_witness);
        self.provenance(&plan.provenance);
        self.executable_aspect_plans(&plan.aspect_plan);
    }

    fn reconcile_plan(&mut self, plan: &ReconcileRecordPlan) {
        self.tag(3);
        self.record_ref(&plan.source_record);
        self.record_ref(&plan.target_record);
        self.visible_snapshot(&plan.source_visible_snapshot);
        self.reconciled_identity_basis(&plan.identity_basis);
        self.merge_record_causal_disposition(plan.causal_disposition);
        self.provenance(&plan.provenance);
        self.executable_aspect_plans(&plan.aspect_plan);
    }

    fn deleted_on_both_sides_plan(&mut self, plan: &ConvergeDeletedOnBothSidesRecordPlan) {
        self.tag(4);
        self.record_ref(&plan.source_record);
        self.optional_record_ref(plan.target_record.as_ref());
        self.shared_truth_witness(&plan.equality_witness);
        self.deleted_on_both_sides_semantics(plan.semantics);
        self.merge_lineage_continuity(plan.lineage_continuity);
        self.provenance(&plan.provenance);
    }

    fn visible_snapshot(&mut self, value: &VisibleMergeRecordSnapshot) {
        match value {
            VisibleMergeRecordSnapshot::Entity(record) => {
                self.tag(1);
                self.str(
                    &crate::query::data::canonical_digest::query_authoritative_entity_record_digest(
                        record,
                    ),
                );
            }
            VisibleMergeRecordSnapshot::Relation(record) => {
                self.tag(2);
                self.str(&crate::query::data::canonical_digest::query_authoritative_relation_record_digest(record));
            }
        }
    }

    fn visible_record_kind(&mut self, value: VisibleMergeRecordKind) {
        match value {
            VisibleMergeRecordKind::Entity => self.tag(1),
            VisibleMergeRecordKind::Relation => self.tag(2),
        }
    }

    pub(super) fn shared_truth_witness(&mut self, value: &SharedTruthWitness) {
        self.str(&value.witness_digest);
    }

    fn reconciled_identity_basis(&mut self, value: &ReconciledIdentityBasis) {
        self.record_ref(&value.source_record);
        self.record_ref(&value.target_record);
    }

    fn executable_aspect_plans(&mut self, values: &[ExecutableAspectPlan]) {
        self.usize(values.len());
        for value in values {
            self.executable_aspect_plan(value);
        }
    }

    fn executable_aspect_plan(&mut self, value: &ExecutableAspectPlan) {
        match value {
            ExecutableAspectPlan::AdoptSourceValue {
                aspect_key,
                source_value,
            } => {
                self.tag(1);
                self.str(aspect_key.as_str());
                self.materialized_value(source_value);
            }
            ExecutableAspectPlan::PreserveSharedValue {
                aspect_key,
                shared_value,
            } => {
                self.tag(2);
                self.str(aspect_key.as_str());
                self.materialized_value(shared_value);
            }
            ExecutableAspectPlan::ReconcileValue {
                aspect_key,
                source_value,
                target_value,
                base_value,
                resolved_value,
            } => {
                self.tag(3);
                self.str(aspect_key.as_str());
                self.optional_materialized_value(source_value.as_ref());
                self.optional_materialized_value(target_value.as_ref());
                self.optional_materialized_value(base_value.as_ref());
                self.optional_materialized_value(resolved_value.as_ref());
            }
        }
    }

    pub(super) fn provenance(&mut self, value: &MergeExecutableRecordProvenance) {
        self.merge_conflict_class(value.classification);
        self.merge_resolution_class(value.resolution_class);
        self.merge_executable_class(value.executable_class);
        self.merge_record_causal_disposition(value.causal_disposition);
        self.merge_policy_proof_boundary(value.policy_proof_boundary);
        self.resolved_aspect_policies(&value.applied_policies);
    }

    fn resolved_aspect_policies(&mut self, values: &[ResolvedAspectMergePolicy]) {
        self.usize(values.len());
        for value in values {
            self.str(value.aspect_key.as_str());
            self.aspect_merge_policy_kind(&value.policy);
        }
    }
}
