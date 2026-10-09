/// Whether an installed request runs as one native attempt or managed bounded steps.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryExecutionBoundary {
    Atomic,
    BoundedStep,
}
impl WorthQueryExecutionBoundary {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Atomic => "atomic",
            Self::BoundedStep => "bounded-step",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryExecutionMode {
    Synchronous,
    Asynchronous,
}

impl WorthQueryExecutionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Synchronous => "synchronous",
            Self::Asynchronous => "asynchronous",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryExecutionDegradation {
    PartialResult,
}

impl WorthQueryExecutionDegradation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PartialResult => "partial-result",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryYieldedStatePosture {
    NotYieldable,
    ProviderCheckpoint,
}

impl WorthQueryYieldedStatePosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotYieldable => "not-yieldable",
            Self::ProviderCheckpoint => "provider-checkpoint",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryRetainedProgressPosture {
    ReleaseAfterAttempt,
    RetainAttemptCapacity,
}

impl WorthQueryRetainedProgressPosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReleaseAfterAttempt => "release-after-attempt",
            Self::RetainAttemptCapacity => "retain-attempt-capacity",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryPartialEffectPosture {
    EffectFree,
    PartialEffectsMayRemain,
}

impl WorthQueryPartialEffectPosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EffectFree => "effect-free",
            Self::PartialEffectsMayRemain => "partial-effects-may-remain",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryCancellationSafePointFamily(String);

impl WorthQueryCancellationSafePointFamily {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err("empty-cancellation-safe-point-family");
        }
        if value.trim() != value || value.chars().any(char::is_control) {
            return Err("invalid-cancellation-safe-point-family");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryCancellationSafePointFamily;

    #[test]
    fn safe_point_families_reject_nonportable_boundaries() {
        assert!(WorthQueryCancellationSafePointFamily::new(" chunk").is_err());
        assert!(WorthQueryCancellationSafePointFamily::new("chunk\nboundary").is_err());
    }
}
