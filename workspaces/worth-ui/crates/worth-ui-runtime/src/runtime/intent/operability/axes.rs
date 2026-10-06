#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentSupportPosture {
    Supported,
    Unsupported,
}

/// The condition an axis reads when it holds no truth value. The slot names
/// the condition in the active expression catalog only, and slots renumber
/// across generations, so it stays inside the runtime; inspection projects
/// the condition's identity from that catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentWithheldCondition {
    slot: crate::runtime::expression::UiExpressionSlot,
    withholding: crate::runtime::expression::UiExpressionWithholding,
}

impl UiIntentWithheldCondition {
    pub(crate) const fn new(
        slot: crate::runtime::expression::UiExpressionSlot,
        withholding: crate::runtime::expression::UiExpressionWithholding,
    ) -> Self {
        Self { slot, withholding }
    }

    pub(crate) const fn slot(self) -> crate::runtime::expression::UiExpressionSlot {
        self.slot
    }

    pub const fn withholding(self) -> crate::runtime::expression::UiExpressionWithholding {
        self.withholding
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentMutabilityPosture {
    Writable,
    Readonly,
    /// A condition source holds no truth value; never read as `Readonly`.
    Withheld(UiIntentWithheldCondition),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentReadinessPosture {
    Ready,
    Pending,
    /// A condition source holds no truth value; never read as `Pending`.
    Withheld(UiIntentWithheldCondition),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentOccupancyPosture {
    Idle,
    InFlight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentPolicyPosture {
    Admitted,
    Denied,
    /// A condition source holds no truth value; never read as `Denied`.
    Withheld(UiIntentWithheldCondition),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentAffinityPosture {
    Current,
    Stale,
    WrongWorld,
    RebindRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIntentConfirmationPosture {
    NotRequired,
    Required { policy_identity: Box<str> },
}

impl UiIntentConfirmationPosture {
    pub fn required_policy_identity(&self) -> Option<&str> {
        match self {
            Self::NotRequired => None,
            Self::Required { policy_identity } => Some(policy_identity),
        }
    }
}
