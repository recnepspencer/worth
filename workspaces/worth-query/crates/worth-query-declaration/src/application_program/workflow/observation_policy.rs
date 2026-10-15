//! Explicit disclosure meaning. A declaration never grants a read.
use super::ApplicationWorkflowSpec;
use crate::application_capability::ApplicationCapabilityMarkerIdentity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Scope whose native facts a governed read capability must disclose.
pub enum ApplicationWorkflowReadScope {
    /// One workflow subject, instance or lineage.
    Subject,
    /// Definition-level metadata independent of an instance subject.
    Definition,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Typed read capability declaration; carries no permission or public read handle.
pub struct ApplicationWorkflowReadRule {
    scope: ApplicationWorkflowReadScope,
    capability: String,
    capability_type: String,
}
impl ApplicationWorkflowReadRule {
    /// Selects a capability for subject-scoped disclosure.
    pub fn subject<Spec, Capability>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema>,
    {
        Self::declared::<Capability>(ApplicationWorkflowReadScope::Subject)
    }
    /// Selects a capability for definition-scoped disclosure.
    pub fn definition<Spec, Capability>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Spec::Schema>,
    {
        Self::declared::<Capability>(ApplicationWorkflowReadScope::Definition)
    }
    fn declared<C: ApplicationCapabilityMarkerIdentity>(
        scope: ApplicationWorkflowReadScope,
    ) -> Self {
        Self {
            scope,
            capability: C::IDENTIFIER.to_owned(),
            capability_type: C::PORTABLE_TYPE_IDENTITY.as_str().to_owned(),
        }
    }
    /// Declared capability name and portable type identity.
    pub fn capability(&self) -> (&str, &str) {
        (&self.capability, &self.capability_type)
    }
    /// Native scope this declaration requires.
    pub const fn scope(&self) -> &ApplicationWorkflowReadScope {
        &self.scope
    }
    /// Current canonical policy identity.
    pub fn persistence_identity(&self) -> String {
        format!(
            "1:{:?}:{}:{}:{}{}",
            self.scope,
            self.capability.len(),
            self.capability_type.len(),
            self.capability,
            self.capability_type
        )
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Explicit applicability of separately governed audit disclosure.
pub enum ApplicationWorkflowAuditPolicy {
    /// No audit disclosure is declared.
    NotApplicable,
    /// Audit reads require their own subject-scoped capability.
    Governed(ApplicationWorkflowReadRule),
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Installed read-policy meaning. A declaration alone never opens a read route.
pub enum ApplicationWorkflowObservationPolicy {
    /// Public observation and discovery are disabled.
    Disabled,
    /// Declared governed reads; routes refuse until their native owner implements enforcement.
    Governed {
        /// Subject-scoped instance disclosure.
        instance: ApplicationWorkflowReadRule,
        /// Definition-scoped metadata disclosure.
        definition: ApplicationWorkflowReadRule,
        /// Independently declared audit disclosure.
        audit: ApplicationWorkflowAuditPolicy,
    },
}
impl ApplicationWorkflowObservationPolicy {
    /// Explicitly closes public observation routes.
    pub const fn disabled() -> Self {
        Self::Disabled
    }
    /// Declares read capabilities; installation validates their scopes and identities.
    pub fn governed(
        /// Subject-scoped instance disclosure.
        instance: ApplicationWorkflowReadRule,
        /// Definition-scoped metadata disclosure.
        definition: ApplicationWorkflowReadRule,
        /// Independently declared audit disclosure.
        audit: ApplicationWorkflowAuditPolicy,
    ) -> Self {
        Self::Governed {
            instance,
            definition,
            audit,
        }
    }
    /// Checks the native subject/definition/audit scope contract.
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Disabled => true,
            Self::Governed {
                instance,
                definition,
                audit,
            } => {
                instance.scope == ApplicationWorkflowReadScope::Subject
                    && definition.scope == ApplicationWorkflowReadScope::Definition
                    && !matches!(audit, ApplicationWorkflowAuditPolicy::Governed(rule) if rule.scope != ApplicationWorkflowReadScope::Subject)
            }
        }
    }
    /// Current canonical policy identity.
    pub fn persistence_identity(&self) -> String {
        match self {
            Self::Disabled => "1:disabled".to_owned(),
            Self::Governed {
                instance,
                definition,
                audit,
            } => {
                let audit = match audit {
                    ApplicationWorkflowAuditPolicy::NotApplicable => "not-applicable".to_owned(),
                    ApplicationWorkflowAuditPolicy::Governed(rule) => rule.persistence_identity(),
                };
                let parts = [
                    instance.persistence_identity(),
                    definition.persistence_identity(),
                    audit,
                ];
                parts.iter().fold("1:governed".to_owned(), |s, p| {
                    format!("{s}:{}:{p}", p.len())
                })
            }
        }
    }
}
