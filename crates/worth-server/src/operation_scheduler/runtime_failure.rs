use worth_query::facade::runtime::WorthQueryRuntimeError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerSchedulerRuntimeFailure {
    DirectMutationAssertionDenied {
        detail: String,
    },
    DirectMutationBindingDenied {
        detail: String,
    },
    DirectMutationContractDenied {
        detail: String,
    },
    DirectMutationContinuityDenied {
        detail: String,
    },
    DirectMutationNamingDenied {
        detail: String,
    },
    DirectMutationTargetReferenceDenied {
        detail: String,
    },
    /// The advancement refused admission or completion of the basis lookup.
    BasisAdvancementDenied {
        denial: worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial,
        detail: String,
    },
    /// The snapshot reader refused custody of an already borrowed phase.
    BasisSnapshotReadDenied {
        denial:
            worth_query_host::facade::application_contribution::WorthQueryForeignAdvancementPhase,
        detail: String,
    },
    Opaque {
        detail: String,
    },
}

impl WorthServerSchedulerRuntimeFailure {
    pub(crate) fn from_mutation_runtime_error(error: WorthQueryRuntimeError) -> Self {
        match error {
            WorthQueryRuntimeError::ExistingTruthAssertionDenied(_) => {
                Self::DirectMutationAssertionDenied {
                    detail: error.to_string(),
                }
            }
            WorthQueryRuntimeError::MutationBindingDenied(_) => Self::DirectMutationBindingDenied {
                detail: error.to_string(),
            },
            WorthQueryRuntimeError::MutationContractDenied(_) => {
                Self::DirectMutationContractDenied {
                    detail: error.to_string(),
                }
            }
            WorthQueryRuntimeError::MutationContinuityDenied(_) => {
                Self::DirectMutationContinuityDenied {
                    detail: error.to_string(),
                }
            }
            WorthQueryRuntimeError::MutationNamingDenied(_) => Self::DirectMutationNamingDenied {
                detail: error.to_string(),
            },
            WorthQueryRuntimeError::MutationTargetReferenceDenied(_) => {
                Self::DirectMutationTargetReferenceDenied {
                    detail: error.to_string(),
                }
            }
            other => Self::Opaque {
                detail: other.to_string(),
            },
        }
    }

    pub(crate) fn opaque(detail: impl Into<String>) -> Self {
        Self::Opaque {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        match self {
            Self::DirectMutationAssertionDenied { detail }
            | Self::DirectMutationBindingDenied { detail }
            | Self::DirectMutationContractDenied { detail }
            | Self::DirectMutationContinuityDenied { detail }
            | Self::DirectMutationNamingDenied { detail }
            | Self::DirectMutationTargetReferenceDenied { detail }
            | Self::BasisAdvancementDenied { detail, .. }
            | Self::BasisSnapshotReadDenied { detail, .. }
            | Self::Opaque { detail } => detail,
        }
    }
}
