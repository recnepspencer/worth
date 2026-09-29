#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiIntentOperabilityContractSpec {
    identity: Box<str>,
    mutability: WorthUiIntentMutabilitySourceSpec,
    readiness: WorthUiIntentReadinessSourceSpec,
    policy: WorthUiIntentPolicySourceSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiIntentMutabilitySourceSpec {
    ApplicationBoolean { fact: Box<str> },
    ProjectionReadonly { projection: Box<str> },
    CommittedDraft,
    Condition { condition: Box<str> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiIntentReadinessSourceSpec {
    ApplicationBoolean { fact: Box<str> },
    Projection { projection: Box<str> },
    CommittedDraft,
    Condition { condition: Box<str> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiIntentPolicySourceSpec {
    ApplicationBoolean { fact: Box<str> },
    Condition { condition: Box<str> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiIntentConfirmationContractSpec {
    policy_identity: Box<str>,
    source: WorthUiIntentConfirmationSourceSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiIntentConfirmationSourceSpec {
    NotRequired,
    ApplicationBoolean { fact: Box<str> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiIntentConcurrencyScope {
    TargetRouteSingleFlight,
    DeclarationSingleFlight,
    DefinitionSingleFlight,
    ApplicationSingleFlight,
}

impl WorthUiIntentOperabilityContractSpec {
    pub fn new(
        identity: impl Into<Box<str>>,
        mutability: WorthUiIntentMutabilitySourceSpec,
        readiness: WorthUiIntentReadinessSourceSpec,
        policy: WorthUiIntentPolicySourceSpec,
    ) -> Self {
        Self {
            identity: required_text(identity, "operability contract identity"),
            mutability,
            readiness,
            policy,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn mutability(&self) -> &WorthUiIntentMutabilitySourceSpec {
        &self.mutability
    }

    pub const fn readiness(&self) -> &WorthUiIntentReadinessSourceSpec {
        &self.readiness
    }

    pub const fn policy(&self) -> &WorthUiIntentPolicySourceSpec {
        &self.policy
    }

    pub(crate) fn revision_token(&self) -> String {
        format!(
            "operability:{}:{}:{}:{}",
            self.identity,
            self.mutability.revision_token(),
            self.readiness.revision_token(),
            self.policy.revision_token()
        )
    }
}

impl WorthUiIntentMutabilitySourceSpec {
    pub fn application_boolean(fact: impl Into<Box<str>>) -> Self {
        Self::ApplicationBoolean {
            fact: required_text(fact, "mutability application fact"),
        }
    }

    pub fn projection_readonly(projection: impl Into<Box<str>>) -> Self {
        Self::ProjectionReadonly {
            projection: required_text(projection, "mutability projection"),
        }
    }

    pub const fn committed_draft() -> Self {
        Self::CommittedDraft
    }

    pub fn condition(condition: impl Into<Box<str>>) -> Self {
        Self::Condition {
            condition: required_text(condition, "mutability condition"),
        }
    }

    pub fn application_fact(&self) -> Option<&str> {
        match self {
            Self::ApplicationBoolean { fact } => Some(fact),
            Self::ProjectionReadonly { .. } | Self::CommittedDraft | Self::Condition { .. } => None,
        }
    }

    pub fn projection(&self) -> Option<&str> {
        match self {
            Self::ProjectionReadonly { projection } => Some(projection),
            Self::ApplicationBoolean { .. } | Self::CommittedDraft | Self::Condition { .. } => None,
        }
    }

    pub fn condition_identity(&self) -> Option<&str> {
        match self {
            Self::Condition { condition } => Some(condition),
            Self::ApplicationBoolean { .. }
            | Self::ProjectionReadonly { .. }
            | Self::CommittedDraft => None,
        }
    }

    pub const fn is_committed_draft(&self) -> bool {
        matches!(self, Self::CommittedDraft)
    }

    fn revision_token(&self) -> String {
        match self {
            Self::ApplicationBoolean { fact } => format!("application-boolean:{fact}"),
            Self::ProjectionReadonly { projection } => {
                format!("projection-readonly:{projection}")
            }
            Self::CommittedDraft => "committed-draft".to_owned(),
            Self::Condition { condition } => format!("condition:{condition}"),
        }
    }
}

impl WorthUiIntentReadinessSourceSpec {
    pub fn application_boolean(fact: impl Into<Box<str>>) -> Self {
        Self::ApplicationBoolean {
            fact: required_text(fact, "readiness application fact"),
        }
    }

    pub fn projection(projection: impl Into<Box<str>>) -> Self {
        Self::Projection {
            projection: required_text(projection, "readiness projection"),
        }
    }

    pub const fn committed_draft() -> Self {
        Self::CommittedDraft
    }

    pub fn condition(condition: impl Into<Box<str>>) -> Self {
        Self::Condition {
            condition: required_text(condition, "readiness condition"),
        }
    }

    pub fn application_fact(&self) -> Option<&str> {
        match self {
            Self::ApplicationBoolean { fact } => Some(fact),
            Self::Projection { .. } | Self::CommittedDraft | Self::Condition { .. } => None,
        }
    }

    pub fn projection_identity(&self) -> Option<&str> {
        match self {
            Self::Projection { projection } => Some(projection),
            Self::ApplicationBoolean { .. } | Self::CommittedDraft | Self::Condition { .. } => None,
        }
    }

    pub fn condition_identity(&self) -> Option<&str> {
        match self {
            Self::Condition { condition } => Some(condition),
            Self::ApplicationBoolean { .. } | Self::Projection { .. } | Self::CommittedDraft => {
                None
            }
        }
    }

    pub const fn is_committed_draft(&self) -> bool {
        matches!(self, Self::CommittedDraft)
    }

    fn revision_token(&self) -> String {
        match self {
            Self::ApplicationBoolean { fact } => format!("application-boolean:{fact}"),
            Self::Projection { projection } => format!("projection:{projection}"),
            Self::CommittedDraft => "committed-draft".to_owned(),
            Self::Condition { condition } => format!("condition:{condition}"),
        }
    }
}

impl WorthUiIntentPolicySourceSpec {
    pub fn application_boolean(fact: impl Into<Box<str>>) -> Self {
        Self::ApplicationBoolean {
            fact: required_text(fact, "policy application fact"),
        }
    }

    pub fn condition(condition: impl Into<Box<str>>) -> Self {
        Self::Condition {
            condition: required_text(condition, "policy condition"),
        }
    }

    pub fn application_fact(&self) -> Option<&str> {
        match self {
            Self::ApplicationBoolean { fact } => Some(fact),
            Self::Condition { .. } => None,
        }
    }

    pub fn condition_identity(&self) -> Option<&str> {
        match self {
            Self::Condition { condition } => Some(condition),
            Self::ApplicationBoolean { .. } => None,
        }
    }

    fn revision_token(&self) -> String {
        match self {
            Self::ApplicationBoolean { fact } => format!("application-boolean:{fact}"),
            Self::Condition { condition } => format!("condition:{condition}"),
        }
    }
}

impl WorthUiIntentConfirmationContractSpec {
    pub fn not_required(policy_identity: impl Into<Box<str>>) -> Self {
        Self {
            policy_identity: required_text(policy_identity, "confirmation policy identity"),
            source: WorthUiIntentConfirmationSourceSpec::NotRequired,
        }
    }

    pub fn application_boolean(
        policy_identity: impl Into<Box<str>>,
        fact: impl Into<Box<str>>,
    ) -> Self {
        Self {
            policy_identity: required_text(policy_identity, "confirmation policy identity"),
            source: WorthUiIntentConfirmationSourceSpec::ApplicationBoolean {
                fact: required_text(fact, "confirmation application fact"),
            },
        }
    }

    pub fn policy_identity(&self) -> &str {
        &self.policy_identity
    }

    pub const fn source(&self) -> &WorthUiIntentConfirmationSourceSpec {
        &self.source
    }

    pub(crate) fn revision_token(&self) -> String {
        match &self.source {
            WorthUiIntentConfirmationSourceSpec::NotRequired => {
                format!("confirmation:{}:not-required", self.policy_identity)
            }
            WorthUiIntentConfirmationSourceSpec::ApplicationBoolean { fact } => {
                format!(
                    "confirmation:{}:application-boolean:{fact}",
                    self.policy_identity
                )
            }
        }
    }
}

impl WorthUiIntentConfirmationSourceSpec {
    pub fn application_fact(&self) -> Option<&str> {
        match self {
            Self::NotRequired => None,
            Self::ApplicationBoolean { fact } => Some(fact),
        }
    }
}

impl WorthUiIntentConcurrencyScope {
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::TargetRouteSingleFlight => "target-route-single-flight",
            Self::DeclarationSingleFlight => "declaration-single-flight",
            Self::DefinitionSingleFlight => "definition-single-flight",
            Self::ApplicationSingleFlight => "application-single-flight",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "target-route-single-flight" => Some(Self::TargetRouteSingleFlight),
            "declaration-single-flight" => Some(Self::DeclarationSingleFlight),
            "definition-single-flight" => Some(Self::DefinitionSingleFlight),
            "application-single-flight" => Some(Self::ApplicationSingleFlight),
            _ => None,
        }
    }
}

fn required_text(value: impl Into<Box<str>>, label: &str) -> Box<str> {
    let value = value.into();
    assert!(!value.trim().is_empty(), "{label} cannot be empty");
    value
}
