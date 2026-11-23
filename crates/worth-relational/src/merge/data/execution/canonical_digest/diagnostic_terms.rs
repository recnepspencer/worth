use crate::merge::data::execution::executable_plan::{
    DeletedOnBothSidesSemantics, MergeLineageContinuityVerdict, SharedTruthWitness,
};
use crate::merge::data::execution_artifacts::{
    ExecutedMergeAspectClass, ExecutedMergeAspectDiagnosticRow, ExecutedMergeRecordClass,
    ExecutedMergeRecordDiagnosticRow,
};

impl super::CanonicalDigestBytes {
    pub(super) fn executed_record_rows(&mut self, values: &[ExecutedMergeRecordDiagnosticRow]) {
        self.usize(values.len());
        for value in values {
            self.executed_record_row(value);
        }
    }

    fn executed_record_row(&mut self, value: &ExecutedMergeRecordDiagnosticRow) {
        self.executed_record_class(value.class);
        self.optional_record_ref(value.source_record.as_ref());
        self.optional_record_ref(value.target_record.as_ref());
        self.optional_record_ref(value.record.as_ref());
        self.optional_shared_truth_witness(value.equality_witness.as_ref());
        self.optional_deleted_on_both_sides_semantics(value.deletion_semantics);
        self.optional_merge_lineage_continuity(value.lineage_continuity);
        self.provenance(&value.provenance);
        self.executed_aspect_rows(&value.aspect_rows);
    }

    fn executed_aspect_rows(&mut self, values: &[ExecutedMergeAspectDiagnosticRow]) {
        self.usize(values.len());
        for value in values {
            self.executed_aspect_row(value);
        }
    }

    fn executed_aspect_row(&mut self, value: &ExecutedMergeAspectDiagnosticRow) {
        self.str(value.aspect_key.as_str());
        self.executed_aspect_class(value.class);
        self.optional_materialized_value(value.source_value.as_ref());
        self.optional_materialized_value(value.target_value.as_ref());
        self.optional_materialized_value(value.base_value.as_ref());
        self.optional_materialized_value(value.shared_value.as_ref());
        self.optional_materialized_value(value.resolved_value.as_ref());
    }

    fn optional_shared_truth_witness(&mut self, value: Option<&SharedTruthWitness>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.shared_truth_witness(value);
            }
            None => self.tag(0),
        }
    }

    fn optional_deleted_on_both_sides_semantics(
        &mut self,
        value: Option<DeletedOnBothSidesSemantics>,
    ) {
        match value {
            Some(value) => {
                self.tag(1);
                self.deleted_on_both_sides_semantics(value);
            }
            None => self.tag(0),
        }
    }

    fn optional_merge_lineage_continuity(&mut self, value: Option<MergeLineageContinuityVerdict>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.merge_lineage_continuity(value);
            }
            None => self.tag(0),
        }
    }

    fn executed_record_class(&mut self, value: ExecutedMergeRecordClass) {
        match value {
            ExecutedMergeRecordClass::AdoptSource => self.tag(1),
            ExecutedMergeRecordClass::PreserveShared => self.tag(2),
            ExecutedMergeRecordClass::Reconcile => self.tag(3),
            ExecutedMergeRecordClass::ConvergeDeletedOnBothSides => self.tag(4),
        }
    }

    fn executed_aspect_class(&mut self, value: ExecutedMergeAspectClass) {
        match value {
            ExecutedMergeAspectClass::AdoptSourceValue => self.tag(1),
            ExecutedMergeAspectClass::PreserveSharedValue => self.tag(2),
            ExecutedMergeAspectClass::ReconcileValue => self.tag(3),
        }
    }
}
