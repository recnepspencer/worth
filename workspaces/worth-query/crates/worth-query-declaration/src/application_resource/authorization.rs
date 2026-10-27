use crate::application_query::ApplicationQueryAuthorizationRequirement;
use crate::application_schema::ApplicationAbilityRef;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationResourceAction {
    Start,
    Inspect,
    Advance,
    Close,
}

impl ApplicationResourceAction {
    pub const ALL: [Self; 4] = [Self::Start, Self::Inspect, Self::Advance, Self::Close];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Inspect => "inspect",
            Self::Advance => "advance",
            Self::Close => "close",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Start => 0,
            Self::Inspect => 1,
            Self::Advance => 2,
            Self::Close => 3,
        }
    }
}

/// Each action is independently authored; payload disclosure is a separate contract.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceAuthorization {
    requirements: [ApplicationQueryAuthorizationRequirement; 4],
}

impl ApplicationResourceAuthorization {
    pub const fn public() -> Self {
        Self {
            requirements: [
                ApplicationQueryAuthorizationRequirement::Public,
                ApplicationQueryAuthorizationRequirement::Public,
                ApplicationQueryAuthorizationRequirement::Public,
                ApplicationQueryAuthorizationRequirement::Public,
            ],
        }
    }

    pub fn require<Schema, Ability, Scope>(
        mut self,
        action: ApplicationResourceAction,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
    ) -> Self {
        self.requirements[action.index()] =
            ApplicationQueryAuthorizationRequirement::for_ability(ability);
        self
    }

    pub fn from_untrusted_requirements(
        requirements: [ApplicationQueryAuthorizationRequirement; 4],
    ) -> Self {
        Self { requirements }
    }

    pub fn requirement(
        &self,
        action: ApplicationResourceAction,
    ) -> &ApplicationQueryAuthorizationRequirement {
        &self.requirements[action.index()]
    }
}
