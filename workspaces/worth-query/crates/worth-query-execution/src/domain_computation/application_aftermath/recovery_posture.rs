//! Durability follows the original owner's actual Home attachment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRecoveryDurabilityPosture {
    ProcessLocal,
    DurableHome,
}
impl WorthQueryRecoveryDurabilityPosture {
    pub const fn as_decision58_label(self) -> &'static str {
        match self {
            Self::ProcessLocal => "process-local",
            Self::DurableHome => "durable-home",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDispatchOutboxDurabilityPosture {
    ProcessLocal,
    DurableHome,
}
impl WorthQueryDispatchOutboxDurabilityPosture {
    pub const fn as_decision58_label(self) -> &'static str {
        match self {
            Self::ProcessLocal => "process-local",
            Self::DurableHome => "durable-home",
        }
    }
}
