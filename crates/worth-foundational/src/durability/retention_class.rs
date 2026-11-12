/// Why an owner retains durable material. Expiry never grants re-execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RetentionClass {
    Authoritative,
    MandatoryObligation,
    OptionalRetainedMaterial,
    Derived,
}

pub trait DurableArtifactRetention {
    const RETENTION_CLASS: RetentionClass;
}
