//! Explicit disclosure meaning. A declaration never grants a read.
use super::ApplicationWorkflowSpec;
use super::{ApplicationWorkflowReadBinding, ApplicationWorkflowReadBindingDeclaration};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Scope whose native facts a governed read capability must disclose.
pub enum ApplicationWorkflowReadScope {
    /// One explicit workflow subject; no implicit lineage or successor expansion.
    Subject,
    /// One explicitly selected definition under Spec and branch, across all subjects.
    Definition,
    /// Metadata of one explicitly selected definition, without instance enumeration.
    DefinitionMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Typed read capability declaration; carries no permission or public read handle.
pub struct ApplicationWorkflowReadRule {
    scope: ApplicationWorkflowReadScope,
    binding: ApplicationWorkflowReadBindingDeclaration,
}
impl ApplicationWorkflowReadRule {
    /// Selects a capability for subject-scoped disclosure.
    pub fn subject<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationWorkflowReadBinding<Spec::Schema>,
    {
        Self::declared::<Spec::Schema, Binding>(ApplicationWorkflowReadScope::Subject)
    }
    /// Selects a capability for definition-scoped disclosure.
    pub fn definition<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationWorkflowReadBinding<Spec::Schema>,
    {
        Self::declared::<Spec::Schema, Binding>(ApplicationWorkflowReadScope::Definition)
    }
    /// Explicit metadata-only disclosure; does not enumerate a definition's instances.
    pub fn definition_metadata<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationWorkflowReadBinding<Spec::Schema>,
    {
        Self::declared::<Spec::Schema, Binding>(ApplicationWorkflowReadScope::DefinitionMetadata)
    }
    fn declared<
        Schema: crate::application_schema::ApplicationSchema,
        B: ApplicationWorkflowReadBinding<Schema>,
    >(
        scope: ApplicationWorkflowReadScope,
    ) -> Self {
        Self {
            scope,
            binding: ApplicationWorkflowReadBindingDeclaration::from_binding::<Schema, B>(),
        }
    }
    pub fn binding(&self) -> &ApplicationWorkflowReadBindingDeclaration {
        &self.binding
    }
    /// Declared capability name and portable type identity.
    pub fn capability(&self) -> (&str, &str) {
        self.binding.capability()
    }
    /// Native scope this declaration requires.
    pub const fn scope(&self) -> &ApplicationWorkflowReadScope {
        &self.scope
    }
    /// Current canonical policy identity.
    pub fn persistence_identity(&self) -> String {
        let extent = match self.scope {
            ApplicationWorkflowReadScope::Subject => "exact-subject",
            ApplicationWorkflowReadScope::Definition => "exact-definition-across-subjects",
            ApplicationWorkflowReadScope::DefinitionMetadata => "exact-definition-metadata",
        };
        format!("2:{extent}:{}", self.binding.persistence_identity())
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
        /// Instances of one explicitly selected definition across subjects.
        definition: ApplicationWorkflowReadRule,
        /// Separate metadata-only route; absence closes this route.
        definition_metadata: Option<ApplicationWorkflowReadRule>,
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
        // Subject-scoped instance disclosure.
        instance: ApplicationWorkflowReadRule,
        // Instances of one explicitly selected definition across subjects.
        definition: ApplicationWorkflowReadRule,
        // Separate metadata-only route; absence closes this route.
        definition_metadata: Option<ApplicationWorkflowReadRule>,
        // Independently declared audit disclosure.
        audit: ApplicationWorkflowAuditPolicy,
    ) -> Self {
        Self::Governed {
            instance,
            definition,
            definition_metadata,
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
                definition_metadata,
            } => {
                instance.scope == ApplicationWorkflowReadScope::Subject
                    && definition.scope == ApplicationWorkflowReadScope::Definition
                    && !matches!(definition_metadata, Some(rule) if rule.scope != ApplicationWorkflowReadScope::DefinitionMetadata)
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
                definition_metadata,
            } => {
                let audit = match audit {
                    ApplicationWorkflowAuditPolicy::NotApplicable => "not-applicable".to_owned(),
                    ApplicationWorkflowAuditPolicy::Governed(rule) => rule.persistence_identity(),
                };
                let parts = [
                    instance.persistence_identity(),
                    definition.persistence_identity(),
                    definition_metadata
                        .as_ref()
                        .map(ApplicationWorkflowReadRule::persistence_identity)
                        .unwrap_or_else(|| "not-applicable".to_owned()),
                    audit,
                ];
                parts.iter().fold("2:governed".to_owned(), |s, p| {
                    format!("{s}:{}:{p}", p.len())
                })
            }
        }
    }
}

/// Closed disclosure route. Definition scope never arises from a missing subject.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowReadRoute {
    Subject,
    Definition,
    DefinitionMetadata,
    Audit,
}
impl ApplicationWorkflowObservationPolicy {
    /// The exact installed declaration for a requested disclosure route.
    pub fn rule(
        &self,
        route: ApplicationWorkflowReadRoute,
    ) -> Option<&ApplicationWorkflowReadRule> {
        match (self, route) {
            (Self::Governed { instance, .. }, ApplicationWorkflowReadRoute::Subject) => {
                Some(instance)
            }
            (Self::Governed { definition, .. }, ApplicationWorkflowReadRoute::Definition) => {
                Some(definition)
            }
            (
                Self::Governed {
                    definition_metadata: Some(rule),
                    ..
                },
                ApplicationWorkflowReadRoute::DefinitionMetadata,
            ) => Some(rule),
            (
                Self::Governed {
                    audit: ApplicationWorkflowAuditPolicy::Governed(rule),
                    ..
                },
                ApplicationWorkflowReadRoute::Audit,
            ) => Some(rule),
            _ => None,
        }
    }
}
