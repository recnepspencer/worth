use crate::installation::IsolatedPulseInstallation;

use super::atomic_replacement::{self, AppliedPulseSourceDelta, PulseSourceActionFailure};
use super::PulseSourceDeltaIdentity;

const READY_RELEASED: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 2,
  "operability": "ready",
  "executor_gate": "released"
}
"#;
const QUERY_DENIAL_REQUESTED: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 2,
  "operability": "ready",
  "executor_gate": "released",
  "query_denial_requested": true
}
"#;
const CONFIRMATION_HELD: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 3,
  "operability": "confirmation_required",
  "executor_gate": "held"
}
"#;
const CONFIRMATION_RELEASED: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 4,
  "operability": "confirmation_required",
  "executor_gate": "released"
}
"#;
const DISABLED: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 5,
  "operability": "disabled",
  "executor_gate": "released"
}
"#;
const DENIED: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 6,
  "operability": "denied",
  "executor_gate": "released"
}
"#;
const FINAL_HELD: &[u8] = br#"{
  "protocol": "worth-ui.platform-pulse.intent-source",
  "schema_version": 1,
  "revision": 7,
  "operability": "ready",
  "executor_gate": "held"
}
"#;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadyReleasedIntentDelta;

#[derive(Clone, Copy, Debug)]
pub(crate) struct QueryDenialRequestedIntentDelta;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConfirmationHeldIntentDelta;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConfirmationReleasedIntentDelta;

#[derive(Clone, Copy, Debug)]
pub(crate) struct DisabledIntentDelta;

#[derive(Clone, Copy, Debug)]
pub(crate) struct DeniedIntentDelta;

#[derive(Clone, Copy, Debug)]
pub(crate) struct FinalHeldIntentDelta;

macro_rules! intent_delta {
    ($kind:ty, $identity:ident, $bytes:ident) => {
        impl $kind {
            pub(crate) fn apply(
                self,
                installation: &IsolatedPulseInstallation,
            ) -> Result<AppliedPulseSourceDelta<Self>, PulseSourceActionFailure> {
                atomic_replacement::apply_path(
                    installation.intent_source(),
                    PulseSourceDeltaIdentity::$identity,
                    $bytes,
                )
            }
        }
    };
}

intent_delta!(
    ReadyReleasedIntentDelta,
    IntentReadyReleased,
    READY_RELEASED
);
intent_delta!(
    QueryDenialRequestedIntentDelta,
    IntentQueryDenialRequested,
    QUERY_DENIAL_REQUESTED
);
intent_delta!(
    ConfirmationHeldIntentDelta,
    IntentConfirmationHeld,
    CONFIRMATION_HELD
);
intent_delta!(
    ConfirmationReleasedIntentDelta,
    IntentConfirmationReleased,
    CONFIRMATION_RELEASED
);
intent_delta!(DisabledIntentDelta, IntentDisabled, DISABLED);
intent_delta!(DeniedIntentDelta, IntentDenied, DENIED);
intent_delta!(FinalHeldIntentDelta, IntentFinalHeld, FINAL_HELD);
