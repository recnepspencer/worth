use super::WorthQueryGraphReadAccessAdmissionPosture;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessDenialKind {
    BudgetExceeded,
    RequiredAsyncMaterialization,
    RequiredAccessCapabilityRegistration,
    RequiredPersistentIndex,
    UnsupportedGraphIndexSupport,
}

impl WorthQueryGraphReadAccessDenialKind {
    pub const ALL: [Self; 5] = [
        Self::BudgetExceeded,
        Self::RequiredAsyncMaterialization,
        Self::RequiredAccessCapabilityRegistration,
        Self::RequiredPersistentIndex,
        Self::UnsupportedGraphIndexSupport,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BudgetExceeded => "budget_exceeded",
            Self::RequiredAsyncMaterialization => "required_async_materialization",
            Self::RequiredAccessCapabilityRegistration => "required_access_capability_registration",
            Self::RequiredPersistentIndex => "required_persistent_index",
            Self::UnsupportedGraphIndexSupport => "unsupported_graph_index_support",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadBudgetExceededDenial {
    max_inline_index_bytes: Option<usize>,
    estimated_index_bytes: usize,
    max_inline_result_bytes: Option<usize>,
    estimated_result_bytes: usize,
    max_inline_intermediate_set_size: Option<usize>,
    estimated_intermediate_set_size: usize,
}

impl WorthQueryGraphReadBudgetExceededDenial {
    pub fn max_inline_index_bytes(&self) -> Option<usize> {
        self.max_inline_index_bytes
    }

    pub fn estimated_index_bytes(&self) -> usize {
        self.estimated_index_bytes
    }

    pub fn max_inline_result_bytes(&self) -> Option<usize> {
        self.max_inline_result_bytes
    }

    pub fn estimated_result_bytes(&self) -> usize {
        self.estimated_result_bytes
    }

    pub fn max_inline_intermediate_set_size(&self) -> Option<usize> {
        self.max_inline_intermediate_set_size
    }

    pub fn estimated_intermediate_set_size(&self) -> usize {
        self.estimated_intermediate_set_size
    }

    pub(crate) fn new(
        max_inline_index_bytes: Option<usize>,
        estimated_index_bytes: usize,
        max_inline_result_bytes: Option<usize>,
        estimated_result_bytes: usize,
        max_inline_intermediate_set_size: Option<usize>,
        estimated_intermediate_set_size: usize,
    ) -> Self {
        Self {
            max_inline_index_bytes,
            estimated_index_bytes,
            max_inline_result_bytes,
            estimated_result_bytes,
            max_inline_intermediate_set_size,
            estimated_intermediate_set_size,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "budget_exceeded_v2:max_index:{:?}:estimated_index:{}:max_result:{:?}:estimated_result:{}:max_intermediate:{:?}:estimated_intermediate:{}",
            self.max_inline_index_bytes,
            self.estimated_index_bytes,
            self.max_inline_result_bytes,
            self.estimated_result_bytes,
            self.max_inline_intermediate_set_size,
            self.estimated_intermediate_set_size
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessDenial {
    kind: WorthQueryGraphReadAccessDenialKind,
    suggested_posture: WorthQueryGraphReadAccessAdmissionPosture,
    budget_exceeded: Option<WorthQueryGraphReadBudgetExceededDenial>,
}

impl WorthQueryGraphReadAccessDenial {
    pub fn kind(&self) -> &WorthQueryGraphReadAccessDenialKind {
        &self.kind
    }

    pub fn suggested_posture(&self) -> &WorthQueryGraphReadAccessAdmissionPosture {
        &self.suggested_posture
    }

    pub fn budget_exceeded(&self) -> Option<&WorthQueryGraphReadBudgetExceededDenial> {
        self.budget_exceeded.as_ref()
    }

    pub(crate) fn from_budget_exceeded(
        budget_exceeded: WorthQueryGraphReadBudgetExceededDenial,
    ) -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::BudgetExceeded,
            suggested_posture:
                WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired,
            budget_exceeded: Some(budget_exceeded),
        }
    }

    pub(crate) fn required_async_materialization() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::RequiredAsyncMaterialization,
            suggested_posture:
                WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired,
            budget_exceeded: None,
        }
    }

    pub(crate) fn required_access_capability_registration() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::RequiredAccessCapabilityRegistration,
            suggested_posture:
                WorthQueryGraphReadAccessAdmissionPosture::AccessCapabilityRegistrationRequired,
            budget_exceeded: None,
        }
    }

    pub(crate) fn required_persistent_index() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::RequiredPersistentIndex,
            suggested_posture: WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired,
            budget_exceeded: None,
        }
    }

    pub(crate) fn unsupported_graph_index_support() -> Self {
        Self {
            kind: WorthQueryGraphReadAccessDenialKind::UnsupportedGraphIndexSupport,
            suggested_posture: WorthQueryGraphReadAccessAdmissionPosture::Denied,
            budget_exceeded: None,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        format!(
            "denial:{}:{}:{}",
            self.kind.as_str(),
            self.suggested_posture.as_str(),
            self.budget_exceeded
                .as_ref()
                .map(WorthQueryGraphReadBudgetExceededDenial::digest_part)
                .unwrap_or_else(|| "budget_exceeded:none".to_string())
        )
    }
}
