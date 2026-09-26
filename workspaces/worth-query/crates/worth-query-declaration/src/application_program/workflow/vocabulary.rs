use crate::application_operation::ApplicationMutationBinding;
use crate::{
    application_capability::ApplicationCapabilityContextEntitySlotBinding,
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

use super::ApplicationWorkflowSpecIdentity;

/// Pure marker for one workflow vocabulary family.
///
/// Installation binds the concrete supported members separately; declaring this
/// marker neither installs nor authorizes any member.
pub trait ApplicationWorkflowSpec: Sized + 'static {
    type Schema: ApplicationSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowOperationRef {
    identifier: &'static str,
    input_type: WorthQueryPortableTypeIdentity,
    operation_type: std::any::TypeId,
    binding: Option<(&'static str, std::any::TypeId, bool)>,
}

impl ApplicationWorkflowOperationRef {
    pub fn declared<Spec, Operation>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Operation: ApplicationOperationMarkerIdentity<Spec::Schema> + 'static,
    {
        Self {
            identifier: Operation::IDENTIFIER,
            input_type: <Operation::InputBinding as crate::application_schema::ApplicationStructuredValueBinding>::IDENTITY,
            operation_type: std::any::TypeId::of::<Operation>(),
            binding: None,
        }
    }

    pub fn declared_binding<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationMutationBinding<Spec::Schema>,
    {
        Self {
            identifier: Binding::Operation::IDENTIFIER,
            input_type: Binding::InputBinding::IDENTITY,
            operation_type: std::any::TypeId::of::<Binding::Operation>(),
            binding: Some((
                Binding::IDENTITY,
                std::any::TypeId::of::<Binding>(),
                Binding::REQUIRES_WORKFLOW_AUTHORITY,
            )),
        }
    }

    pub const fn identifier(&self) -> &'static str {
        self.identifier
    }

    pub const fn input_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.input_type
    }

    #[doc(hidden)]
    pub const fn operation_type(&self) -> std::any::TypeId {
        self.operation_type
    }

    pub const fn binding(&self) -> Option<(&'static str, std::any::TypeId, bool)> {
        self.binding
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowAssessmentRef {
    identifier: &'static str,
    parameter_type: WorthQueryPortableTypeIdentity,
    result_type: WorthQueryPortableTypeIdentity,
    query_type: std::any::TypeId,
    subject: ApplicationWorkflowSubjectSelector,
    applicability: ApplicationWorkflowAssessmentApplicability,
}

/// Authored membership of one potential assessment in the required inventory.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowAssessmentApplicability {
    Always,
    WhenRelatedRelationPresent {
        relation: &'static str,
        from: &'static str,
        to: &'static str,
    },
}

impl ApplicationWorkflowAssessmentApplicability {
    pub const fn relation(&self) -> Option<(&'static str, &'static str, &'static str)> {
        match self {
            Self::Always => None,
            Self::WhenRelatedRelationPresent { relation, from, to } => Some((relation, from, to)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationWorkflowSubjectSelector {
    Resource,
    Related,
    Context(ApplicationCapabilityContextEntitySlotBinding),
}

impl ApplicationWorkflowSubjectSelector {
    pub const fn resource() -> Self {
        Self::Resource
    }

    pub const fn related() -> Self {
        Self::Related
    }

    pub fn context(slot: ApplicationCapabilityContextEntitySlotBinding) -> Self {
        Self::Context(slot)
    }

    pub const fn context_slot(&self) -> Option<&ApplicationCapabilityContextEntitySlotBinding> {
        match self {
            Self::Context(slot) => Some(slot),
            Self::Resource | Self::Related => None,
        }
    }

    #[doc(hidden)]
    pub fn persistence_identity(&self) -> String {
        match self {
            Self::Resource => "resource".to_owned(),
            Self::Related => "related".to_owned(),
            Self::Context(slot) => {
                let fields = [
                    slot.context(),
                    slot.context_identity_ref().as_str(),
                    slot.slot(),
                    slot.slot_identity_ref().as_str(),
                    slot.entity(),
                ];
                fields
                    .iter()
                    .fold("context".to_owned(), |mut encoded, field| {
                        use std::fmt::Write;
                        write!(&mut encoded, "|{}:{field}", field.len())
                            .expect("writing workflow subject identity cannot fail");
                        encoded
                    })
            }
        }
    }

    #[doc(hidden)]
    pub fn from_persistence_identity(identity: &str) -> Option<Self> {
        match identity {
            "resource" => Some(Self::Resource),
            "related" => Some(Self::Related),
            _ => {
                let encoded = identity.strip_prefix("context|")?;
                let mut fields = Vec::with_capacity(5);
                let mut rest = encoded;
                while fields.len() < 5 {
                    let colon = rest.find(':')?;
                    let length = rest[..colon].parse::<usize>().ok()?;
                    let value_start = colon.checked_add(1)?;
                    let value_end = value_start.checked_add(length)?;
                    let value = rest.get(value_start..value_end)?;
                    fields.push(value.to_owned());
                    rest = rest.get(value_end..)?;
                    if fields.len() < 5 {
                        rest = rest.strip_prefix('|')?;
                    }
                }
                if !rest.is_empty() {
                    return None;
                }
                Some(Self::Context(
                    ApplicationCapabilityContextEntitySlotBinding::from_untrusted_parts(
                        crate::application_capability::WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
                            context: fields.remove(0),
                            context_identity: WorthQueryPortableTypeIdentity::from_untrusted(fields.remove(0)),
                            slot: fields.remove(0),
                            slot_identity: WorthQueryPortableTypeIdentity::from_untrusted(fields.remove(0)),
                            entity: fields.remove(0),
                        },
                    ),
                ))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowConditionRef {
    identifier: &'static str,
    parameter_type: WorthQueryPortableTypeIdentity,
    result_type: WorthQueryPortableTypeIdentity,
    query_type: std::any::TypeId,
}

impl ApplicationWorkflowConditionRef {
    pub fn declared<Spec, Query>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
        Query::ResultBinding:
            crate::application_schema::ApplicationStructuredValueBinding<Value = bool>,
    {
        Self {
            identifier: Query::IDENTIFIER,
            parameter_type: Query::PARAMETER_TYPE_IDENTITY,
            result_type: Query::RESULT_TYPE_IDENTITY,
            query_type: std::any::TypeId::of::<Query>(),
        }
    }

    pub const fn identifier(&self) -> &'static str {
        self.identifier
    }

    pub const fn parameter_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parameter_type
    }

    pub const fn result_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.result_type
    }

    #[doc(hidden)]
    pub const fn query_type(&self) -> std::any::TypeId {
        self.query_type
    }
}

impl ApplicationWorkflowAssessmentRef {
    pub fn declared<Spec, Query>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        Self::declared_for::<Spec, Query>(ApplicationWorkflowSubjectSelector::Resource)
    }

    pub fn declared_for<Spec, Query>(subject: ApplicationWorkflowSubjectSelector) -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        Self {
            identifier: Query::IDENTIFIER,
            parameter_type: Query::PARAMETER_TYPE_IDENTITY,
            result_type: Query::RESULT_TYPE_IDENTITY,
            query_type: std::any::TypeId::of::<Query>(),
            subject,
            applicability: ApplicationWorkflowAssessmentApplicability::Always,
        }
    }

    pub fn declared_when_related_relation_present<Spec, Query, Relation, From, To>(
        relation: crate::application_schema::ApplicationRelationRef<
            Spec::Schema,
            Relation,
            From,
            To,
        >,
    ) -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        let mut assessment =
            Self::declared_for::<Spec, Query>(ApplicationWorkflowSubjectSelector::Related);
        assessment.applicability =
            ApplicationWorkflowAssessmentApplicability::WhenRelatedRelationPresent {
                relation: relation.name(),
                from: relation.from(),
                to: relation.to(),
            };
        assessment
    }

    pub const fn subject(&self) -> &ApplicationWorkflowSubjectSelector {
        &self.subject
    }

    pub const fn applicability(&self) -> &ApplicationWorkflowAssessmentApplicability {
        &self.applicability
    }

    pub const fn identifier(&self) -> &'static str {
        self.identifier
    }

    pub const fn parameter_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parameter_type
    }

    pub const fn result_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.result_type
    }

    #[doc(hidden)]
    pub const fn query_type(&self) -> std::any::TypeId {
        self.query_type
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowApprovalRef {
    identifier: &'static str,
    capability_type: WorthQueryPortableTypeIdentity,
    marker_type: std::any::TypeId,
}

impl ApplicationWorkflowApprovalRef {
    pub fn declared<Spec, Capability>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema> + 'static,
    {
        Self {
            identifier: Capability::IDENTIFIER,
            capability_type: Capability::PORTABLE_TYPE_IDENTITY,
            marker_type: std::any::TypeId::of::<Capability>(),
        }
    }

    pub const fn identifier(&self) -> &'static str {
        self.identifier
    }

    pub const fn capability_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.capability_type
    }

    #[doc(hidden)]
    pub const fn marker_type(&self) -> std::any::TypeId {
        self.marker_type
    }
}
