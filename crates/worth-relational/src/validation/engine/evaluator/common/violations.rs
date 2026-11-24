use crate::diagnostics::data::DiagnosticCode;
use crate::validation::data::execution::InvariantClass;
use crate::validation::data::results::InvariantViolation;
use crate::validation::data::results::InvariantViolationFields;
use crate::validation::data::results::StorageInconsistencyFailure;
use crate::validation::data::results::StorageInconsistencyLookup;
use crate::validation::data::results::StorageInconsistencyScan;
use worth_foundational::facade::FieldKey;

#[derive(Debug, Clone, Default)]
pub(crate) struct StorageInconsistencyContext {
    entity_id: Option<crate::identity::data::EntityId>,
    partition_id: Option<crate::identity::data::PartitionId>,
    slot: Option<usize>,
    field: Option<FieldKey>,
    missing_label: Option<String>,
    scan: Option<StorageInconsistencyScan>,
    lookup: Option<StorageInconsistencyLookup>,
    failure: Option<StorageInconsistencyFailure>,
}

impl StorageInconsistencyContext {
    pub(crate) fn with_entity_id(mut self, entity_id: crate::identity::data::EntityId) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    pub(crate) fn with_partition_id(
        mut self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Self {
        self.partition_id = Some(partition_id);
        self
    }

    pub(crate) fn with_field(mut self, field: FieldKey) -> Self {
        self.field = Some(field);
        self
    }

    pub(crate) fn with_scan(mut self, scan: StorageInconsistencyScan) -> Self {
        self.scan = Some(scan);
        self
    }

    pub(crate) fn with_lookup(mut self, lookup: StorageInconsistencyLookup) -> Self {
        self.lookup = Some(lookup);
        self
    }

    pub(crate) fn with_failure(mut self, failure: StorageInconsistencyFailure) -> Self {
        self.failure = Some(failure);
        self
    }
}

pub(crate) fn canonicalize_violations(
    mut violations: Vec<InvariantViolation>,
) -> Vec<InvariantViolation> {
    violations.sort_by_key(|violation| violation.witness_key());
    violations
}

pub(crate) fn relation_violation(
    class: InvariantClass,
    code: DiagnosticCode,
    detail: String,
    fields: InvariantViolationFields,
) -> InvariantViolation {
    InvariantViolation {
        class,
        code,
        detail,
        fields,
    }
}

pub(crate) fn storage_inconsistency_violation(
    class: InvariantClass,
    detail: String,
    context: StorageInconsistencyContext,
) -> InvariantViolation {
    InvariantViolation {
        class,
        code: DiagnosticCode::StorageInconsistencyDetected,
        detail,
        fields: InvariantViolationFields::StorageInconsistency {
            entity_id: context.entity_id,
            partition_id: context.partition_id,
            slot: context.slot,
            field: context.field,
            missing_label: context.missing_label,
            scan: context.scan,
            lookup: context.lookup,
            failure: context.failure,
        },
    }
}
