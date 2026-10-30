use worth_foundational::facade::RetentionDeliveryProfile;

use crate::domain_computation::WorthQueryArtifactClassification;

macro_rules! portable_identity {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
                let value = value.into();
                if !super::validation::portable_identity(&value) {
                    return Err("invalid-portable-decision-identity");
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

portable_identity!(WorthQueryDecisionKind);
portable_identity!(WorthQueryDecisionReasonFamily);
portable_identity!(WorthQueryArtifactKeyFamily);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryDecisionPayloadVersion(u32);

impl WorthQueryDecisionPayloadVersion {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryDecisionCausalParentShape {
    None,
    OptionalSingle,
    RequiredSingle,
    OrderedMany,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionIdentity {
    kind: WorthQueryDecisionKind,
    reason_family: WorthQueryDecisionReasonFamily,
    affected_artifact_key_family: WorthQueryArtifactKeyFamily,
}

impl WorthQueryDecisionIdentity {
    pub fn new(
        kind: WorthQueryDecisionKind,
        reason_family: WorthQueryDecisionReasonFamily,
        affected_artifact_key_family: WorthQueryArtifactKeyFamily,
    ) -> Self {
        Self {
            kind,
            reason_family,
            affected_artifact_key_family,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionGovernance {
    classification: WorthQueryArtifactClassification,
    retention: RetentionDeliveryProfile,
}

impl WorthQueryDecisionGovernance {
    pub const fn new(
        classification: WorthQueryArtifactClassification,
        retention: RetentionDeliveryProfile,
    ) -> Self {
        Self {
            classification,
            retention,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionSchema {
    kind: WorthQueryDecisionKind,
    reason_family: WorthQueryDecisionReasonFamily,
    affected_artifact_key_family: WorthQueryArtifactKeyFamily,
    causal_parent: WorthQueryDecisionCausalParentShape,
    payload_version: WorthQueryDecisionPayloadVersion,
    classification: WorthQueryArtifactClassification,
    retention: RetentionDeliveryProfile,
}

impl WorthQueryDecisionSchema {
    pub fn new(
        identity: WorthQueryDecisionIdentity,
        causal_parent: WorthQueryDecisionCausalParentShape,
        payload_version: WorthQueryDecisionPayloadVersion,
        governance: WorthQueryDecisionGovernance,
    ) -> Self {
        Self {
            kind: identity.kind,
            reason_family: identity.reason_family,
            affected_artifact_key_family: identity.affected_artifact_key_family,
            causal_parent,
            payload_version,
            classification: governance.classification,
            retention: governance.retention,
        }
    }

    pub fn kind(&self) -> &WorthQueryDecisionKind {
        &self.kind
    }

    pub fn reason_family(&self) -> &WorthQueryDecisionReasonFamily {
        &self.reason_family
    }

    pub fn affected_artifact_key_family(&self) -> &WorthQueryArtifactKeyFamily {
        &self.affected_artifact_key_family
    }

    pub const fn causal_parent(&self) -> WorthQueryDecisionCausalParentShape {
        self.causal_parent
    }

    pub const fn payload_version(&self) -> WorthQueryDecisionPayloadVersion {
        self.payload_version
    }

    pub const fn classification(&self) -> WorthQueryArtifactClassification {
        self.classification
    }

    pub const fn retention(&self) -> RetentionDeliveryProfile {
        self.retention
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDecisionRecordContract {
    NotRequired,
    Declared {
        schemas: Vec<WorthQueryDecisionSchema>,
    },
}

impl WorthQueryDecisionRecordContract {
    pub const fn not_required() -> Self {
        Self::NotRequired
    }

    pub fn declared(schemas: impl IntoIterator<Item = WorthQueryDecisionSchema>) -> Self {
        let mut schemas = schemas.into_iter().collect::<Vec<_>>();
        schemas.sort_by(|left, right| left.kind.cmp(&right.kind));
        Self::Declared { schemas }
    }

    pub fn schemas(&self) -> &[WorthQueryDecisionSchema] {
        match self {
            Self::NotRequired => &[],
            Self::Declared { schemas } => schemas,
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        super::validation::contract_is_valid(self)
    }
}
