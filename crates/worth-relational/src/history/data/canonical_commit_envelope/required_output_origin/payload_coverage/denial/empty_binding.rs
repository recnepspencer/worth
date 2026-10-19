//! Contract context captured only where empty native birth payload is refused.
use super::super::{charge, OutputPayloadCoverageFailure};
use super::BindingForm;
use crate::schema::data::LoweredAspectContractBinding;
use worth_execution::MapKernelContext;
use worth_foundational::facade::{AbsenceLaw, AspectShape, FieldRequirement};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super::super) struct EmptyBindingSummary {
    ordinal: usize,
    identity: u64,
    revision: u64,
    target: BindingForm,
    shape: ShapeForm,
    absence: AbsenceLaw,
    declared: usize,
    required: usize,
    optional: usize,
    defaulted: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShapeForm {
    Scalar,
    Struct,
    Opaque,
    Reference,
    Content,
}
impl ShapeForm {
    fn label(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Struct => "struct",
            Self::Opaque => "opaque",
            Self::Reference => "reference",
            Self::Content => "content",
        }
    }
}

impl EmptyBindingSummary {
    pub(in super::super) fn capture(
        ordinal: usize,
        binding: &LoweredAspectContractBinding,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Self, OutputPayloadCoverageFailure> {
        charge(context, 1)?;
        let mut summary = Self {
            ordinal,
            identity: binding.contract.identity().0,
            revision: binding.contract.revision().0,
            target: BindingForm::capture(&binding.target),
            shape: match binding.contract.shape() {
                AspectShape::Scalar(_) => ShapeForm::Scalar,
                AspectShape::Struct(_) => ShapeForm::Struct,
                AspectShape::Opaque(_) => ShapeForm::Opaque,
                AspectShape::Reference(_) => ShapeForm::Reference,
                AspectShape::Content => ShapeForm::Content,
            },
            absence: binding.contract.absence(),
            declared: 0,
            required: 0,
            optional: 0,
            defaulted: 0,
        };
        if let AspectShape::Struct(shape) = binding.contract.shape() {
            charge(context, shape.fields().len())?;
            summary.declared = shape.fields().len();
            for field in shape.fields() {
                match field.requirement() {
                    FieldRequirement::Required => summary.required += 1,
                    FieldRequirement::Optional => summary.optional += 1,
                    FieldRequirement::Defaulted => summary.defaulted += 1,
                }
            }
        }
        Ok(summary)
    }

    pub(super) fn detail(self) -> String {
        let absence = match self.absence {
            AbsenceLaw::Required => "required",
            AbsenceLaw::Optional => "optional",
            AbsenceLaw::Defaulted => "defaulted",
        };
        format!(
            "binding_ordinal={} binding_identity={} binding_revision={} binding={} shape={} absence={} declared_fields={} required_fields={} optional_fields={} defaulted_fields={}",
            self.ordinal, self.identity, self.revision, self.target.label(), self.shape.label(),
            absence, self.declared, self.required, self.optional, self.defaulted
        )
    }
}
