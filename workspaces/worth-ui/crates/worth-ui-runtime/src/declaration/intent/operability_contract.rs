#[path = "operability_contract/resolution.rs"]
mod resolution;

pub(crate) use resolution::resolve_operability_contract;

use crate::capability::UiIntentBoolean;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentOperabilityDependencyAxis {
    Mutability,
    Readiness,
    Policy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentOperabilityContractIdentityError {
    InvalidIdentity,
}

pub struct UiIntentOperabilityContract {
    identity: Box<str>,
    mutability: UiIntentMutabilitySource,
    readiness: UiIntentReadinessSource,
    policy: UiIntentPolicySource,
}

pub struct UiIntentMutabilitySource {
    source: UiAuthoredIntentMutabilitySource,
}

enum UiAuthoredIntentMutabilitySource {
    ApplicationBoolean(Box<str>),
    ProjectionReadonly(Box<str>),
    CommittedDraft,
    Condition(Box<str>),
}

pub struct UiIntentReadinessSource {
    source: UiAuthoredIntentReadinessSource,
}

enum UiAuthoredIntentReadinessSource {
    ApplicationBoolean(Box<str>),
    Projection(Box<str>),
    CommittedDraft,
    Condition(Box<str>),
}

pub struct UiIntentPolicySource {
    source: UiAuthoredIntentPolicySource,
}

enum UiAuthoredIntentPolicySource {
    ApplicationBoolean(Box<str>),
    Condition(Box<str>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedIntentOperabilityContract {
    identity: Box<str>,
    mutability: UiResolvedIntentMutabilitySource,
    readiness: UiResolvedIntentReadinessSource,
    policy: UiResolvedIntentPolicySource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiResolvedIntentMutabilitySource {
    ApplicationBoolean(super::UiIntentApplicationFactSlot),
    ProjectionReadonly {
        identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    },
    CommittedDraft,
    Condition(super::UiResolvedIntentExpressionSource),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiResolvedIntentReadinessSource {
    ApplicationBoolean(super::UiIntentApplicationFactSlot),
    Projection {
        identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    },
    CommittedDraft,
    Condition(super::UiResolvedIntentExpressionSource),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiResolvedIntentPolicySource {
    ApplicationBoolean(super::UiIntentApplicationFactSlot),
    Condition(super::UiResolvedIntentExpressionSource),
}

impl UiIntentOperabilityContract {
    pub fn new(
        identity: impl Into<Box<str>>,
        mutability: UiIntentMutabilitySource,
        readiness: UiIntentReadinessSource,
        policy: UiIntentPolicySource,
    ) -> Result<Self, UiIntentOperabilityContractIdentityError> {
        let identity = identity.into();
        if !super::valid_intent_identity(&identity) {
            return Err(UiIntentOperabilityContractIdentityError::InvalidIdentity);
        }
        Ok(Self {
            identity,
            mutability,
            readiness,
            policy,
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) fn into_dsl(self) -> worth_ui_dsl::WorthUiIntentOperabilityContractSpec {
        worth_ui_dsl::WorthUiIntentOperabilityContractSpec::new(
            self.identity,
            self.mutability.into_dsl(),
            self.readiness.into_dsl(),
            self.policy.into_dsl(),
        )
    }
}

impl UiIntentMutabilitySource {
    pub fn application_fact(fact: &super::UiIntentApplicationFact<UiIntentBoolean>) -> Self {
        Self {
            source: UiAuthoredIntentMutabilitySource::ApplicationBoolean(fact.identity().into()),
        }
    }

    pub fn readonly_projection(
        projection: &worth_ui_query_binding::WorthUiQueryViewIdentity,
    ) -> Self {
        Self {
            source: UiAuthoredIntentMutabilitySource::ProjectionReadonly(
                projection.as_str().into(),
            ),
        }
    }

    pub const fn committed_draft() -> Self {
        Self {
            source: UiAuthoredIntentMutabilitySource::CommittedDraft,
        }
    }

    /// Mutability read from the authored condition named `identity`.
    pub fn condition(
        identity: impl Into<Box<str>>,
    ) -> Result<Self, UiIntentOperabilityContractIdentityError> {
        Ok(Self {
            source: UiAuthoredIntentMutabilitySource::Condition(condition_identity(identity)?),
        })
    }

    fn into_dsl(self) -> worth_ui_dsl::WorthUiIntentMutabilitySourceSpec {
        match self.source {
            UiAuthoredIntentMutabilitySource::ApplicationBoolean(fact) => {
                worth_ui_dsl::WorthUiIntentMutabilitySourceSpec::application_boolean(fact)
            }
            UiAuthoredIntentMutabilitySource::ProjectionReadonly(projection) => {
                worth_ui_dsl::WorthUiIntentMutabilitySourceSpec::projection_readonly(projection)
            }
            UiAuthoredIntentMutabilitySource::CommittedDraft => {
                worth_ui_dsl::WorthUiIntentMutabilitySourceSpec::committed_draft()
            }
            UiAuthoredIntentMutabilitySource::Condition(condition) => {
                worth_ui_dsl::WorthUiIntentMutabilitySourceSpec::condition(condition)
            }
        }
    }
}

impl UiIntentReadinessSource {
    pub fn application_fact(fact: &super::UiIntentApplicationFact<UiIntentBoolean>) -> Self {
        Self {
            source: UiAuthoredIntentReadinessSource::ApplicationBoolean(fact.identity().into()),
        }
    }

    pub fn projection(projection: &worth_ui_query_binding::WorthUiQueryViewIdentity) -> Self {
        Self {
            source: UiAuthoredIntentReadinessSource::Projection(projection.as_str().into()),
        }
    }

    pub const fn committed_draft() -> Self {
        Self {
            source: UiAuthoredIntentReadinessSource::CommittedDraft,
        }
    }

    /// Readiness read from the authored condition named `identity`.
    pub fn condition(
        identity: impl Into<Box<str>>,
    ) -> Result<Self, UiIntentOperabilityContractIdentityError> {
        Ok(Self {
            source: UiAuthoredIntentReadinessSource::Condition(condition_identity(identity)?),
        })
    }

    fn into_dsl(self) -> worth_ui_dsl::WorthUiIntentReadinessSourceSpec {
        match self.source {
            UiAuthoredIntentReadinessSource::ApplicationBoolean(fact) => {
                worth_ui_dsl::WorthUiIntentReadinessSourceSpec::application_boolean(fact)
            }
            UiAuthoredIntentReadinessSource::Projection(projection) => {
                worth_ui_dsl::WorthUiIntentReadinessSourceSpec::projection(projection)
            }
            UiAuthoredIntentReadinessSource::CommittedDraft => {
                worth_ui_dsl::WorthUiIntentReadinessSourceSpec::committed_draft()
            }
            UiAuthoredIntentReadinessSource::Condition(condition) => {
                worth_ui_dsl::WorthUiIntentReadinessSourceSpec::condition(condition)
            }
        }
    }
}

impl UiIntentPolicySource {
    pub fn application_fact(fact: &super::UiIntentApplicationFact<UiIntentBoolean>) -> Self {
        Self {
            source: UiAuthoredIntentPolicySource::ApplicationBoolean(fact.identity().into()),
        }
    }

    /// Policy read from the authored condition named `identity`.
    pub fn condition(
        identity: impl Into<Box<str>>,
    ) -> Result<Self, UiIntentOperabilityContractIdentityError> {
        Ok(Self {
            source: UiAuthoredIntentPolicySource::Condition(condition_identity(identity)?),
        })
    }

    fn into_dsl(self) -> worth_ui_dsl::WorthUiIntentPolicySourceSpec {
        match self.source {
            UiAuthoredIntentPolicySource::ApplicationBoolean(fact) => {
                worth_ui_dsl::WorthUiIntentPolicySourceSpec::application_boolean(fact)
            }
            UiAuthoredIntentPolicySource::Condition(condition) => {
                worth_ui_dsl::WorthUiIntentPolicySourceSpec::condition(condition)
            }
        }
    }
}

impl UiResolvedIntentOperabilityContract {
    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn mutability(&self) -> &UiResolvedIntentMutabilitySource {
        &self.mutability
    }

    pub(crate) const fn readiness(&self) -> &UiResolvedIntentReadinessSource {
        &self.readiness
    }

    pub(crate) const fn policy(&self) -> &UiResolvedIntentPolicySource {
        &self.policy
    }

    /// Every condition this contract reads, in axis order.
    pub(crate) fn conditions(
        &self,
    ) -> impl Iterator<Item = &super::UiResolvedIntentExpressionSource> {
        let mutability = match &self.mutability {
            UiResolvedIntentMutabilitySource::Condition(condition) => Some(condition),
            UiResolvedIntentMutabilitySource::ApplicationBoolean(_)
            | UiResolvedIntentMutabilitySource::ProjectionReadonly { .. }
            | UiResolvedIntentMutabilitySource::CommittedDraft => None,
        };
        let readiness = match &self.readiness {
            UiResolvedIntentReadinessSource::Condition(condition) => Some(condition),
            UiResolvedIntentReadinessSource::ApplicationBoolean(_)
            | UiResolvedIntentReadinessSource::Projection { .. }
            | UiResolvedIntentReadinessSource::CommittedDraft => None,
        };
        let policy = match &self.policy {
            UiResolvedIntentPolicySource::Condition(condition) => Some(condition),
            UiResolvedIntentPolicySource::ApplicationBoolean(_) => None,
        };
        mutability.into_iter().chain(readiness).chain(policy)
    }
}

fn condition_identity(
    identity: impl Into<Box<str>>,
) -> Result<Box<str>, UiIntentOperabilityContractIdentityError> {
    let identity = identity.into();
    if super::valid_intent_identity(&identity) {
        Ok(identity)
    } else {
        Err(UiIntentOperabilityContractIdentityError::InvalidIdentity)
    }
}
