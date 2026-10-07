use worth_query::facade::runtime::{
    WorthQueryLowerRuntimeSupportPosture, WorthQueryRuntimePublicApiFamilyContract,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerQuerySupportPosture {
    ProductIndependent {
        label: String,
    },
    PrimaryGraphApplicationSupported {
        basis_token: String,
    },
    QueryReadSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    DirectReadSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    DirectStateSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    DirectInspectionSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    DirectProjectionSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    DirectMutationSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    QueryMutationSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
    },
    DownstreamDeliverySupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
        runtime_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
        durable_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
        contract_digest: String,
    },
    RuntimeBackedResumeSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
        runtime_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
        support_digest: String,
        contract_digest: String,
    },
    DurableResumeSupported {
        family_contract: WorthQueryRuntimePublicApiFamilyContract,
        durable_resume_support_posture: WorthQueryLowerRuntimeSupportPosture,
        support_digest: String,
        contract_digest: String,
    },
}

impl WorthServerQuerySupportPosture {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        match self {
            Self::ProductIndependent { label } => string(label),
            Self::PrimaryGraphApplicationSupported { basis_token } => string(basis_token),
            _ => u64::MAX,
        }
    }
    pub fn runtime_resume_support_posture(
        &self,
    ) -> worth_query::facade::runtime::WorthQueryLowerRuntimeSupportPosture {
        match self {
            Self::ProductIndependent { .. } | Self::PrimaryGraphApplicationSupported { .. } => {
                worth_query::facade::runtime::WorthQueryLowerRuntimeSupportPosture::Forbidden
            }
            Self::DownstreamDeliverySupported {
                runtime_resume_support_posture,
                ..
            }
            | Self::RuntimeBackedResumeSupported {
                runtime_resume_support_posture,
                ..
            } => *runtime_resume_support_posture,
            Self::DurableResumeSupported { .. }
            | Self::QueryReadSupported { .. }
            | Self::DirectReadSupported { .. }
            | Self::DirectStateSupported { .. }
            | Self::DirectInspectionSupported { .. }
            | Self::DirectProjectionSupported { .. }
            | Self::DirectMutationSupported { .. }
            | Self::QueryMutationSupported { .. } => {
                worth_query::facade::runtime::WorthQueryLowerRuntimeSupportPosture::Forbidden
            }
        }
    }

    pub fn durable_resume_support_posture(
        &self,
    ) -> worth_query::facade::runtime::WorthQueryLowerRuntimeSupportPosture {
        match self {
            Self::ProductIndependent { .. } | Self::PrimaryGraphApplicationSupported { .. } => {
                worth_query::facade::runtime::WorthQueryLowerRuntimeSupportPosture::Forbidden
            }
            Self::DownstreamDeliverySupported {
                durable_resume_support_posture,
                ..
            }
            | Self::DurableResumeSupported {
                durable_resume_support_posture,
                ..
            } => *durable_resume_support_posture,
            Self::RuntimeBackedResumeSupported { .. }
            | Self::QueryReadSupported { .. }
            | Self::DirectReadSupported { .. }
            | Self::DirectStateSupported { .. }
            | Self::DirectInspectionSupported { .. }
            | Self::DirectProjectionSupported { .. }
            | Self::DirectMutationSupported { .. }
            | Self::QueryMutationSupported { .. } => {
                worth_query::facade::runtime::WorthQueryLowerRuntimeSupportPosture::Forbidden
            }
        }
    }

    pub(crate) fn canonical_label(&self) -> String {
        match self {
            Self::ProductIndependent { label } => {
                format!("product-independent:{label}")
            }
            Self::PrimaryGraphApplicationSupported { basis_token } => {
                format!("primary-graph-application-supported:{basis_token}")
            }
            Self::QueryReadSupported { family_contract } => {
                format!("query-read-supported:{}", family_contract.contract_digest())
            }
            Self::DirectReadSupported { family_contract } => {
                format!(
                    "direct-read-supported:{}",
                    family_contract.contract_digest()
                )
            }
            Self::DirectStateSupported { family_contract } => {
                format!(
                    "direct-state-supported:{}",
                    family_contract.contract_digest()
                )
            }
            Self::DirectInspectionSupported { family_contract } => format!(
                "direct-inspection-supported:{}",
                family_contract.contract_digest()
            ),
            Self::DirectProjectionSupported { family_contract } => format!(
                "direct-projection-supported:{}",
                family_contract.contract_digest()
            ),
            Self::DirectMutationSupported { family_contract } => {
                format!(
                    "direct-mutation-supported:{}",
                    family_contract.contract_digest()
                )
            }
            Self::QueryMutationSupported { family_contract } => {
                format!(
                    "query-mutation-supported:{}",
                    family_contract.contract_digest()
                )
            }
            Self::DownstreamDeliverySupported {
                family_contract,
                runtime_resume_support_posture,
                durable_resume_support_posture,
                contract_digest,
            } => format!(
                "downstream-delivery-supported:{}:{}:{}:{contract_digest}",
                family_contract.contract_digest(),
                runtime_resume_support_posture.as_str(),
                durable_resume_support_posture.as_str(),
            ),
            Self::RuntimeBackedResumeSupported {
                family_contract,
                runtime_resume_support_posture,
                support_digest,
                contract_digest,
            } => format!(
                "runtime-backed-resume-supported:{}:{}:{support_digest}:{contract_digest}",
                family_contract.contract_digest(),
                runtime_resume_support_posture.as_str(),
            ),
            Self::DurableResumeSupported {
                family_contract,
                durable_resume_support_posture,
                support_digest,
                contract_digest,
            } => format!(
                "durable-resume-supported:{}:{}:{support_digest}:{contract_digest}",
                family_contract.contract_digest(),
                durable_resume_support_posture.as_str(),
            ),
        }
    }
}
