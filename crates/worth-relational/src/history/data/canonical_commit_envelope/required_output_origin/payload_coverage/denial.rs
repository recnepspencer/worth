//! Failure-site summaries contain only fixed labels and native numeric metadata.
mod empty_binding;
pub(super) use empty_binding::EmptyBindingSummary;

use super::{Operation, Value};
use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::schema::data::LoweredAspectContractPlan;
use worth_foundational::facade::{
    AspectBinding, ContractValidatedAspectValue, ContractValidatedAspectValueView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum UnsupportedPayload {
    OpaqueRecord,
    EmptyWithOperations(OperationSummary),
    EmptyWithBindings {
        count: usize,
        first: EmptyBindingSummary,
    },
    EntityWithoutPlan,
    DuplicateOperations(Box<DuplicateSummary>),
    TargetBinding {
        present: PresentSummary,
        operation: OperationSummary,
    },
    OperationValue {
        present: PresentSummary,
        operation: OperationSummary,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) struct DuplicateSummary {
    pub(super) present: PresentSummary,
    pub(super) first: OperationSummary,
    pub(super) second: OperationSummary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PresentForm {
    Scalar,
    Struct,
}
impl PresentForm {
    fn label(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Struct => "struct",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperationForm {
    WholeScalarSet,
    WholeStructSet,
    WholeClear,
    FieldPatch,
}
impl OperationForm {
    fn label(self) -> &'static str {
        match self {
            Self::WholeScalarSet => "whole-scalar-set",
            Self::WholeStructSet => "whole-struct-set",
            Self::WholeClear => "whole-clear",
            Self::FieldPatch => "field-patch",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BindingForm {
    EntityField,
    RelationField,
    RelationSource,
    RelationTarget,
    Region,
    Partition,
    Facet,
    Lifecycle,
    Unknown,
}
impl BindingForm {
    fn capture(binding: &AspectBinding) -> Self {
        match binding {
            AspectBinding::EntityField { .. } => Self::EntityField,
            AspectBinding::RelationField { .. } => Self::RelationField,
            AspectBinding::RelationSourceEndpoint => Self::RelationSource,
            AspectBinding::RelationTargetEndpoint => Self::RelationTarget,
            AspectBinding::StructuralRegion => Self::Region,
            AspectBinding::StructuralPartition => Self::Partition,
            AspectBinding::StructuralFacet => Self::Facet,
            AspectBinding::LifecycleTransition => Self::Lifecycle,
            _ => Self::Unknown,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::EntityField => "entity-field",
            Self::RelationField => "relation-field",
            Self::RelationSource => "relation-source",
            Self::RelationTarget => "relation-target",
            Self::Region => "structural-region",
            Self::Partition => "structural-partition",
            Self::Facet => "structural-facet",
            Self::Lifecycle => "lifecycle",
            Self::Unknown => "unknown-binding",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) struct PresentSummary {
    ordinal: usize,
    identity: u64,
    revision: u64,
    form: PresentForm,
    fields: usize,
}
impl PresentSummary {
    pub(super) fn capture(ordinal: usize, present: &ContractValidatedAspectValue) -> Self {
        let (form, fields) = match present.view() {
            ContractValidatedAspectValueView::Scalar(_) => (PresentForm::Scalar, 0),
            ContractValidatedAspectValueView::Struct(value) => (PresentForm::Struct, value.len()),
        };
        Self {
            ordinal,
            identity: present.contract_identity().0,
            revision: present.contract_revision().0,
            form,
            fields,
        }
    }
    fn detail(self) -> String {
        format!(
            "aspect_ordinal={} aspect_identity={} aspect_revision={} present={} present_fields={}",
            self.ordinal,
            self.identity,
            self.revision,
            self.form.label(),
            self.fields
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) struct OperationSummary {
    ordinal: usize,
    identity: u64,
    revision: u64,
    form: OperationForm,
    binding: BindingForm,
    sets: usize,
    clears: usize,
}
impl OperationSummary {
    pub(super) fn capture(ordinal: usize, operation: &Operation) -> Self {
        let (identity, revision, binding, form, sets, clears) = match operation {
            Operation::WholeAspectSet {
                aspect_identity,
                contract_revision,
                binding,
                value,
                ..
            } => (
                *aspect_identity,
                *contract_revision,
                binding,
                match value {
                    Value::Scalar(_) => OperationForm::WholeScalarSet,
                    Value::Struct(_) => OperationForm::WholeStructSet,
                },
                1,
                0,
            ),
            Operation::WholeAspectClear {
                aspect_identity,
                contract_revision,
                binding,
                ..
            } => (
                *aspect_identity,
                *contract_revision,
                binding,
                OperationForm::WholeClear,
                0,
                1,
            ),
            Operation::FieldLevelPatch {
                aspect_identity,
                contract_revision,
                binding,
                field_sets,
                field_clears,
                ..
            } => (
                *aspect_identity,
                *contract_revision,
                binding,
                OperationForm::FieldPatch,
                field_sets.len(),
                field_clears.len(),
            ),
        };
        Self {
            ordinal,
            identity: identity.0,
            revision: revision.0,
            form,
            binding: BindingForm::capture(binding),
            sets,
            clears,
        }
    }
    fn detail(self) -> String {
        format!("operation_ordinal={} operation_identity={} operation_revision={} operation={} binding={} sets={} clears={}",
            self.ordinal, self.identity, self.revision, self.form.label(), self.binding.label(), self.sets, self.clears)
    }
}

impl UnsupportedPayload {
    fn detail(self) -> String {
        match self {
            Self::OpaqueRecord => "reason=opaque-record".into(),
            Self::EmptyWithOperations(operation) => format!(
                "reason=empty-payload-with-operations {}",
                operation.detail()
            ),
            Self::EmptyWithBindings { count, first } => {
                format!(
                    "reason=empty-payload-with-executable-bindings bindings={count} {}",
                    first.detail()
                )
            }
            Self::EntityWithoutPlan => "reason=entity-without-admitted-plan".into(),
            Self::DuplicateOperations(summary) => format!(
                "reason=duplicate-present-aspect-operations {} first=[{}] second=[{}]",
                summary.present.detail(),
                summary.first.detail(),
                summary.second.detail()
            ),
            Self::TargetBinding { present, operation } => format!(
                "reason=unsupported-target-binding {} {}",
                present.detail(),
                operation.detail()
            ),
            Self::OperationValue { present, operation } => format!(
                "reason=unsupported-operation-value-form {} {}",
                present.detail(),
                operation.detail()
            ),
        }
    }

    pub(in super::super) fn entity_detail(
        self,
        header: &'static str,
        envelope: &CanonicalCommitEnvelope,
        id: EntityId,
        plan: &LoweredAspectContractPlan,
        member_ordinal: Option<usize>,
    ) -> String {
        let mut detail = format!(
            "{header}; record=entity:{}:{}:{} kind={} schema_version={} plan_revision={} executable_bindings={} {}",
            id.partition_value(), id.local_slot_value(), id.generation_value(), plan.kind_id.0,
            envelope.schema_version.0, plan.plan_revision.0, plan.executable_bindings.len(), self.detail());
        if let Some(ordinal) = member_ordinal {
            use std::fmt::Write;
            write!(detail, " member_ordinal={ordinal}").expect("String formatting is infallible");
        }
        detail
    }

    pub(in super::super) fn relation_detail(
        self,
        envelope: &CanonicalCommitEnvelope,
        id: RelationId,
        kind: KindId,
    ) -> String {
        format!("native output relation has unsupported present cells; record=relation:{}:{}:{} kind={} schema_version={} {}",
            id.partition_value(), id.local_slot_value(), id.generation_value(), kind.0,
            envelope.schema_version.0, self.detail())
    }
}
