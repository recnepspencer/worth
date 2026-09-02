use worth_ui::facade::intent::{UiIntentApplicationFact, UiIntentBoolean};

const MUTABILITY: &str = "phase3.operability.writable";
const READINESS: &str = "phase3.operability.ready";
const POLICY: &str = "phase3.operability.policy";
const CONFIRMATION: &str = "phase3.operability.confirmation";

#[derive(Clone)]
pub(in crate::intent) struct OperabilityFacts {
    pub(in crate::intent) mutability: UiIntentApplicationFact<UiIntentBoolean>,
    pub(in crate::intent) readiness: UiIntentApplicationFact<UiIntentBoolean>,
    pub(in crate::intent) policy: UiIntentApplicationFact<UiIntentBoolean>,
    pub(in crate::intent) confirmation: UiIntentApplicationFact<UiIntentBoolean>,
}

impl OperabilityFacts {
    pub(in crate::intent) fn new() -> Self {
        Self {
            mutability: boolean_fact(MUTABILITY),
            readiness: boolean_fact(READINESS),
            policy: boolean_fact(POLICY),
            confirmation: boolean_fact(CONFIRMATION),
        }
    }
}

fn boolean_fact(identity: &str) -> UiIntentApplicationFact<UiIntentBoolean> {
    UiIntentApplicationFact::boolean(identity).expect("operability fact identity is valid")
}
