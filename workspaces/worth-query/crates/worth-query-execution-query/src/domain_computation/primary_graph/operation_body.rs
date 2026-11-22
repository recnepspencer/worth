//! Execution of a complete installed body, never a caller-selected replacement.
//! Pure body value models come from `worth-query-execution-lineage`; handlers,
//! runtime assessment, and candidate application stay in `worth-query-execution`.

pub use worth_query_execution_lineage::facade::primary_graph::operation_body::{
    assessment, children, emission, evaluate_requirements, expression, field_expectation,
    field_value, input, output, path_links, path_value, prepare_inputs, reference_input,
    requirement, retirement, reuse, selection, source, value, BodyFieldExpectation,
    CreationDisposition, PreparedBodyChildren, PreparedBodyInputs, PreparedBodyPathLinks,
    PreparedBodyPathWrites, PreparedBodyRetirements, PreparedBodySelection, PreparedBodySelections,
    PreparedBodySource, PreparedRequirementOperand, PreparedRetirementBatch,
    WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial,
    WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
    WorthQueryOperationBodyRequirementFinding, WorthQueryOperationBodyRequirementStatus,
    WorthQueryOperationBodyRequirementTarget,
};

mod handler;
mod input_expectation;
mod installation;
mod runtime_assessment;

pub use handler::OperationBodyHandler;
