use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema},
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
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationWorkflowAssessmentRef {
    identifier: &'static str,
    parameter_type: WorthQueryPortableTypeIdentity,
    result_type: WorthQueryPortableTypeIdentity,
    query_type: std::any::TypeId,
}

impl ApplicationWorkflowAssessmentRef {
    pub fn declared<Spec, Query>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
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
