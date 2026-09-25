//! Bounded, descriptive source-native dependencies retained with one approval.

use serde::{Deserialize, Serialize};

use super::{
    WorkflowApprovalRequestEncodingDenial, WorthQueryDurableAuthorizationDependencies,
    WorthQueryRetainedCapabilityAuthorization,
};

const VERSION: u8 = 1;
// Each nested Relational stamp may use its full bounded wire budget; this
// envelope has its own aggregate limit rather than the request-text limit.
const MAXIMUM_APPROVAL_DEPENDENCY_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation) struct WorthQueryWorkflowApprovalDependencies {
    version: u8,
    primary: WorthQueryDurableAuthorizationDependencies,
    support: Option<WorthQueryDurableAuthorizationDependencies>,
}

pub(in crate::domain_computation) fn encode_workflow_approval_dependencies(
    authorization: &WorthQueryRetainedCapabilityAuthorization,
) -> Result<String, WorkflowApprovalRequestEncodingDenial> {
    let dependencies = WorthQueryWorkflowApprovalDependencies {
        version: VERSION,
        primary: authorization
            .decision()
            .retained_durable_dependencies()
            .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?,
        support: authorization
            .supporting()
            .map(|support| support.decision().retained_durable_dependencies())
            .transpose()
            .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?,
    };
    let encoded = serde_json::to_string(&dependencies)
        .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?;
    (encoded.len() <= MAXIMUM_APPROVAL_DEPENDENCY_BYTES)
        .then_some(encoded)
        .ok_or(WorkflowApprovalRequestEncodingDenial::TooLarge)
}

impl WorthQueryWorkflowApprovalDependencies {
    pub(in crate::domain_computation) fn decode(encoded: &str) -> Result<Self, ()> {
        if encoded.len() > MAXIMUM_APPROVAL_DEPENDENCY_BYTES {
            return Err(());
        }
        let dependencies: Self = serde_json::from_str(encoded).map_err(|_| ())?;
        if dependencies.version != VERSION {
            return Err(());
        }
        dependencies.primary.validate()?;
        if let Some(support) = &dependencies.support {
            support.validate()?;
        }
        Ok(dependencies)
    }

    pub(in crate::domain_computation) fn support_present(&self) -> bool {
        self.support.is_some()
    }

    pub(in crate::domain_computation) fn primary_matches(
        &self,
        current: &super::WorthQueryAuthorizationDecisionFact,
    ) -> bool {
        current
            .retained_durable_dependencies()
            .is_ok_and(|observed| observed == self.primary)
    }

    pub(in crate::domain_computation) fn support_matches(
        &self,
        current: &super::WorthQueryAuthorizationDecisionFact,
    ) -> bool {
        self.support.as_ref().is_some_and(|expected| {
            current
                .retained_durable_dependencies()
                .is_ok_and(|observed| observed == *expected)
        })
    }
}
