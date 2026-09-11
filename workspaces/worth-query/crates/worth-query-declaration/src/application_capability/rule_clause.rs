use crate::application_schema::ApplicationAuthorizationPath;
use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationFieldUnit,
    DeclaredApplicationFieldValue, WritePosture,
};
use worth_foundational::facade::AspectValue;

use super::{ApplicationCapabilityFieldBinding, ApplicationCapabilityPathContextAnchor};

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableApplicationCapabilityAcceptedValuesParts,
    WorthQueryPortableApplicationCapabilityGraphClauseParts,
    WorthQueryPortableApplicationCapabilityGraphRequirementParts,
    WorthQueryPortableApplicationCapabilityGraphRuleParts,
    WorthQueryPortableApplicationCapabilityScopeGuardParts,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityAcceptedValues {
    field: ApplicationCapabilityFieldBinding,
    values: Vec<AspectValue>,
}

impl ApplicationCapabilityAcceptedValues {
    pub fn one_of<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        values: impl IntoIterator<Item = ApplicationEncodedScalarValue<Field::Binding>>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let mut values = values
            .into_iter()
            .map(ApplicationEncodedScalarValue::into_foundational_value)
            .collect::<Vec<_>>();
        values.sort();
        values.dedup();
        Self {
            field: ApplicationCapabilityFieldBinding::from_reference(field),
            values,
        }
    }

    pub const fn field(&self) -> &ApplicationCapabilityFieldBinding {
        &self.field
    }

    pub fn values(&self) -> &[AspectValue] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityScopeGuard {
    requirements: Vec<ApplicationCapabilityAcceptedValues>,
}

impl ApplicationCapabilityScopeGuard {
    pub const fn unconditional() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }

    pub fn requiring(
        requirements: impl IntoIterator<Item = ApplicationCapabilityAcceptedValues>,
    ) -> Self {
        let mut requirements = requirements.into_iter().collect::<Vec<_>>();
        requirements.sort();
        requirements.dedup();
        Self { requirements }
    }

    pub fn requirements(&self) -> &[ApplicationCapabilityAcceptedValues] {
        &self.requirements
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityGraphClause {
    path: ApplicationAuthorizationPath,
    guard: ApplicationCapabilityScopeGuard,
    context_anchors: Vec<ApplicationCapabilityPathContextAnchor>,
}

impl ApplicationCapabilityGraphClause {
    pub fn new(path: ApplicationAuthorizationPath) -> Self {
        Self {
            path,
            guard: ApplicationCapabilityScopeGuard::unconditional(),
            context_anchors: Vec::new(),
        }
    }

    pub fn when(
        path: ApplicationAuthorizationPath,
        requirements: impl IntoIterator<Item = ApplicationCapabilityAcceptedValues>,
    ) -> Self {
        Self {
            path,
            guard: ApplicationCapabilityScopeGuard::requiring(requirements),
            context_anchors: Vec::new(),
        }
    }

    pub fn requiring(
        mut self,
        requirements: impl IntoIterator<Item = ApplicationCapabilityAcceptedValues>,
    ) -> Self {
        self.guard = ApplicationCapabilityScopeGuard::requiring(requirements);
        self
    }

    pub fn anchored(
        mut self,
        anchors: impl IntoIterator<Item = ApplicationCapabilityPathContextAnchor>,
    ) -> Self {
        self.context_anchors = anchors.into_iter().collect();
        self.context_anchors.sort();
        self.context_anchors.dedup();
        self
    }

    pub const fn path(&self) -> &ApplicationAuthorizationPath {
        &self.path
    }

    pub const fn guard(&self) -> &ApplicationCapabilityScopeGuard {
        &self.guard
    }

    pub fn context_anchors(&self) -> &[ApplicationCapabilityPathContextAnchor] {
        &self.context_anchors
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityGraphRequirement {
    clauses: Vec<ApplicationCapabilityGraphClause>,
}

impl ApplicationCapabilityGraphRequirement {
    pub fn any(clauses: impl IntoIterator<Item = ApplicationCapabilityGraphClause>) -> Self {
        let mut clauses = clauses.into_iter().collect::<Vec<_>>();
        clauses.sort();
        clauses.dedup();
        Self { clauses }
    }

    pub fn clauses(&self) -> &[ApplicationCapabilityGraphClause] {
        &self.clauses
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationCapabilityGraphRule {
    requirements: Vec<ApplicationCapabilityGraphRequirement>,
}

impl ApplicationCapabilityGraphRule {
    pub fn any(clauses: impl IntoIterator<Item = ApplicationCapabilityGraphClause>) -> Self {
        Self {
            requirements: vec![ApplicationCapabilityGraphRequirement::any(clauses)],
        }
    }

    pub fn all(
        requirements: impl IntoIterator<Item = ApplicationCapabilityGraphRequirement>,
    ) -> Self {
        let mut requirements = requirements.into_iter().collect::<Vec<_>>();
        requirements.sort();
        requirements.dedup();
        Self { requirements }
    }

    pub fn requirements(&self) -> &[ApplicationCapabilityGraphRequirement] {
        &self.requirements
    }
}
