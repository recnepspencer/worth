use crate::basis::ExecutionPreflightBundle;
use crate::identity::{BasisDigest, PlanDigest, ResultDigest, ValidatedQueryDigest};

mod preflight;

pub(crate) use preflight::execute_preflight_bundle;
#[cfg(test)]
pub(crate) use preflight::execute_serial_fallback_route;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionCounters {
    execution_read_operation_count: usize,
    execution_records_examined_count: usize,
    execution_records_emitted_count: usize,
    execution_fallback_taken_count: usize,
    execution_result_shape_binding_count: usize,
    page_width: usize,
    page_truncation_count: usize,
    cursor_advance_count: usize,
    post_read_shape_field_count: usize,
    materialized_relation_count: usize,
    aggregate_input_count: usize,
    rollup_input_count: usize,
    derived_field_evaluation_count: usize,
    cdc_output_count: usize,
    executor_semantic_rediscovery_count: usize,
}

impl ExecutionCounters {
    pub fn execution_read_operation_count(&self) -> usize {
        self.execution_read_operation_count
    }

    pub fn execution_records_examined_count(&self) -> usize {
        self.execution_records_examined_count
    }

    pub fn execution_records_emitted_count(&self) -> usize {
        self.execution_records_emitted_count
    }

    pub fn execution_fallback_taken_count(&self) -> usize {
        self.execution_fallback_taken_count
    }

    pub fn execution_result_shape_binding_count(&self) -> usize {
        self.execution_result_shape_binding_count
    }

    pub fn page_width(&self) -> usize {
        self.page_width
    }

    pub fn page_truncation_count(&self) -> usize {
        self.page_truncation_count
    }

    pub fn cursor_advance_count(&self) -> usize {
        self.cursor_advance_count
    }

    pub fn post_read_shape_field_count(&self) -> usize {
        self.post_read_shape_field_count
    }

    pub fn materialized_relation_count(&self) -> usize {
        self.materialized_relation_count
    }

    pub fn aggregate_input_count(&self) -> usize {
        self.aggregate_input_count
    }

    pub fn rollup_input_count(&self) -> usize {
        self.rollup_input_count
    }

    pub fn derived_field_evaluation_count(&self) -> usize {
        self.derived_field_evaluation_count
    }

    pub fn cdc_output_count(&self) -> usize {
        self.cdc_output_count
    }

    pub fn executor_semantic_rediscovery_count(&self) -> usize {
        self.executor_semantic_rediscovery_count
    }

    pub(crate) fn with_materialized_rows(
        mut self,
        records_examined_count: usize,
        row_count: usize,
    ) -> Self {
        self.execution_records_examined_count = records_examined_count;
        self.execution_records_emitted_count = row_count;
        self
    }

    pub(crate) fn with_count_aggregate_input(
        mut self,
        records_examined_count: usize,
        input_row_count: usize,
    ) -> Self {
        self.execution_records_examined_count = records_examined_count;
        self.execution_records_emitted_count = 1;
        self.page_width = 1;
        self.aggregate_input_count = input_row_count;
        self.rollup_input_count = input_row_count;
        self
    }

    #[cfg(test)]
    pub(crate) fn absorb(&mut self, other: &Self) {
        self.execution_read_operation_count += other.execution_read_operation_count;
        self.execution_records_examined_count += other.execution_records_examined_count;
        self.execution_records_emitted_count += other.execution_records_emitted_count;
        self.execution_fallback_taken_count += other.execution_fallback_taken_count;
        self.execution_result_shape_binding_count += other.execution_result_shape_binding_count;
        self.page_width += other.page_width;
        self.page_truncation_count += other.page_truncation_count;
        self.cursor_advance_count += other.cursor_advance_count;
        self.post_read_shape_field_count += other.post_read_shape_field_count;
        self.materialized_relation_count += other.materialized_relation_count;
        self.aggregate_input_count += other.aggregate_input_count;
        self.rollup_input_count += other.rollup_input_count;
        self.derived_field_evaluation_count += other.derived_field_evaluation_count;
        self.cdc_output_count += other.cdc_output_count;
        self.executor_semantic_rediscovery_count += other.executor_semantic_rediscovery_count;
    }

    pub(crate) fn from_preflight(preflight: &ExecutionPreflightBundle) -> Self {
        let read_surfaces = preflight.plan().counters().planned_read_surface_count();
        let route = preflight.plan().query().route();
        let examined = preflight.plan().query().projection_count()
            + preflight.plan().query().traversal_count()
            + preflight.plan().query().predicate_count()
            + preflight.plan().query().ordering_count();
        let emitted = preflight.plan().result_shape().binding_count();
        let collection = preflight.plan().collection();
        Self {
            execution_read_operation_count: match route {
                crate::planning::PlannedExecutionRoute::RuntimeSnapshotRead
                | crate::planning::PlannedExecutionRoute::StoreSnapshotRead => 1,
                crate::planning::PlannedExecutionRoute::RuntimeExpandedSnapshotRead => {
                    read_surfaces.max(1)
                }
            },
            execution_records_examined_count: examined.max(1),
            execution_records_emitted_count: emitted.max(1),
            execution_fallback_taken_count: usize::from(matches!(
                preflight.plan().query().fallback(),
                crate::planning::FallbackDisposition::AdmittedAndSelected
            )),
            execution_result_shape_binding_count: emitted,
            page_width: emitted,
            page_truncation_count: 0,
            cursor_advance_count: usize::from(collection.is_some()),
            post_read_shape_field_count: emitted
                + collection
                    .map(|collection| {
                        collection
                            .post_read_shaping()
                            .derived_field_plan()
                            .derived_field_count()
                    })
                    .unwrap_or(0),
            materialized_relation_count: collection
                .map(|collection| collection.traversal_bound().edge_classes().len())
                .unwrap_or(0),
            aggregate_input_count: collection
                .map(|collection| {
                    collection
                        .post_read_shaping()
                        .aggregate_shape()
                        .input_breadth()
                        .value()
                })
                .unwrap_or(0),
            rollup_input_count: collection
                .map(|collection| {
                    usize::from(!matches!(
                        collection.post_read_shaping().rollup_shape().edge_class(),
                        crate::collection::RollupEdgeClass::NoneAdmittedYet
                    ))
                })
                .unwrap_or(0),
            derived_field_evaluation_count: collection
                .map(|collection| {
                    collection
                        .post_read_shaping()
                        .derived_field_plan()
                        .derived_field_count()
                })
                .unwrap_or(0),
            cdc_output_count: collection
                .map(|collection| {
                    usize::from(matches!(
                        collection.post_read_shaping().result_family(),
                        crate::collection::CollectionResultFamily::CdcCollection
                    ))
                })
                .unwrap_or(0),
            executor_semantic_rediscovery_count: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionReport {
    query_digest: ValidatedQueryDigest,
    plan_digest: PlanDigest,
    basis_digest: BasisDigest,
    result_digest: ResultDigest,
}

impl ExecutionReport {
    pub fn query_digest(&self) -> &ValidatedQueryDigest {
        &self.query_digest
    }

    pub fn plan_digest(&self) -> &PlanDigest {
        &self.plan_digest
    }

    pub fn basis_digest(&self) -> &BasisDigest {
        &self.basis_digest
    }

    pub fn result_digest(&self) -> &ResultDigest {
        &self.result_digest
    }

    pub(crate) fn from_preflight(
        preflight: &ExecutionPreflightBundle,
        result_digest: ResultDigest,
    ) -> Self {
        Self {
            query_digest: preflight.plan().query().validated_query_digest().clone(),
            plan_digest: preflight.plan().query().plan_digest().clone(),
            basis_digest: preflight.basis().proof().digest().clone(),
            result_digest,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionFailureClass {
    UnsupportedExecutionShape,
    InternalInvariantBreak,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionError {
    ExecutionInvariantViolation { message: &'static str },
}

impl ExecutionError {
    pub fn failure_class(&self) -> ExecutionFailureClass {
        match self {
            Self::ExecutionInvariantViolation { .. } => {
                ExecutionFailureClass::InternalInvariantBreak
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionResultEnvelope {
    rows: Vec<String>,
    report: ExecutionReport,
    counters: ExecutionCounters,
}

impl ExecutionResultEnvelope {
    pub fn rows(&self) -> &[String] {
        &self.rows
    }

    pub fn report(&self) -> &ExecutionReport {
        &self.report
    }

    pub fn counters(&self) -> &ExecutionCounters {
        &self.counters
    }

    pub fn check_invariants(&self) -> Result<(), ExecutionError> {
        if self.counters.executor_semantic_rediscovery_count() != 0 {
            return Err(ExecutionError::ExecutionInvariantViolation {
                message: "executor semantic rediscovery must remain zero",
            });
        }

        if self.counters.execution_records_emitted_count() != self.rows.len() {
            return Err(ExecutionError::ExecutionInvariantViolation {
                message: "execution emitted count does not match rows length",
            });
        }

        Ok(())
    }

    pub(crate) fn new(
        rows: Vec<String>,
        report: ExecutionReport,
        counters: ExecutionCounters,
    ) -> Result<Self, ExecutionError> {
        let envelope = Self {
            rows,
            report,
            counters,
        };
        envelope.check_invariants()?;
        Ok(envelope)
    }
}
