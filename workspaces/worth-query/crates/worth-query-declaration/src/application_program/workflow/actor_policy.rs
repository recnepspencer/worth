//! Closed typed bindings selecting native recorded occurrences for policy.
use super::ApplicationWorkflowSpec;
use crate::application_capability::{ApplicationWorkflowRecordedActorSource, ApplicationCapabilityMarkerIdentity};
use crate::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
};
use crate::application_schema::ApplicationOperationMarkerIdentity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Installed operation/binding identity paired with its original native source.
/// Construction requires a typed mutation binding.
pub struct ApplicationWorkflowActorBinding {
    operation: String,
    binding: String,
    source: ApplicationWorkflowRecordedActorSource,
    capability: String,
    capability_type: String,
}
impl ApplicationWorkflowActorBinding {
    /// Exact capability declaration selected by this typed mutation binding.
    pub fn capability(&self) -> (&str, &str) { (&self.capability, &self.capability_type) }

    /// The declared operation identifier.
    pub fn operation(&self) -> &str {
        &self.operation
    }
    /// The exact mutation binding identifier.
    pub fn binding(&self) -> &str {
        &self.binding
    }
    /// The original occurrence selected by the native owner.
    pub const fn source(&self) -> ApplicationWorkflowRecordedActorSource {
        self.source
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Explicit typed closure of workflow recorded-actor capability requirements.
/// Installation checks every binding, installed action, capability and source.
pub enum ApplicationWorkflowActorPolicy {
    /// No command in this vocabulary uses a recorded-actor requirement.
    NotApplicable,
    /// Nonempty exact bindings; duplicate or unenforced entries refuse installation.
    Bound(Vec<ApplicationWorkflowActorBinding>),
}
impl ApplicationWorkflowActorPolicy {
    /// Explicitly declares that no recorded-actor rule applies.
    pub const fn not_applicable() -> Self {
        Self::NotApplicable
    }
    /// Binds a genuine operation to its native original occurrence source.
    pub fn for_operation<Spec, Binding>(source: ApplicationWorkflowRecordedActorSource) -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationCapabilityMutationBinding<Spec::Schema>,
    {
        Self::Bound(vec![ApplicationWorkflowActorBinding {
            operation: Binding::Operation::IDENTIFIER.to_owned(),
            binding: Binding::IDENTITY.to_owned(),
            source,
            capability: Binding::Capability::IDENTIFIER.to_owned(),
            capability_type: Binding::Capability::PORTABLE_TYPE_IDENTITY.as_str().to_owned(),
        }])
    }
    /// Binds a genuine approval command to its native original occurrence source.
    pub fn for_approval<Spec, Binding>(source: ApplicationWorkflowRecordedActorSource) -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationCapabilityMutationBinding<Spec::Schema>,
    {
        Self::for_operation::<Spec, Binding>(source)
    }
    /// Combines typed bindings; installation rejects duplicates.
    pub fn and(self, other: Self) -> Self {
        let mut bindings = match self {
            Self::NotApplicable => Vec::new(),
            Self::Bound(bindings) => bindings,
        };
        bindings.extend(match other {
            Self::NotApplicable => Vec::new(),
            Self::Bound(bindings) => bindings,
        });
        bindings.sort();
        Self::Bound(bindings)
    }
    /// The closed declaration entries, without actor values or authority.
    pub fn bindings(&self) -> &[ApplicationWorkflowActorBinding] {
        match self {
            Self::NotApplicable => &[],
            Self::Bound(values) => values,
        }
    }
    /// Current canonical policy identity used by installation and portable definitions.
    pub fn persistence_identity(&self) -> String {
        match self {
            Self::NotApplicable => "1:not-applicable".to_owned(),
            Self::Bound(values) => values.iter().fold("1:bound".to_owned(), |s, value| {
                format!(
                    "{s}:{}:{}:{}{}:{:?}",
                    value.operation.len(),
                    value.binding.len(),
                    value.operation,
                    value.binding,
                    value.source
                ) + &format!(":{}:{}:{}{}", value.capability.len(), value.capability_type.len(), value.capability, value.capability_type)
            }),
        }
    }
}
