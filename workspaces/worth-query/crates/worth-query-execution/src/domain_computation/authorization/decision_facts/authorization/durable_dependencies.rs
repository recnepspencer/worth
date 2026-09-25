//! Portable descriptions of Relational-owned native authorization dependencies.

use serde::{Deserialize, Serialize};
use worth_relational::facade::authorization::{
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationObservationEvidence,
};
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryAuthorizationDecisionFact;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation) struct WorthQueryDurableAuthorizationDependencies {
    primary: String,
    delegation: Option<WorthQueryDurableDelegationDependencies>,
    activation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum WorthQueryDurableDelegationDependencies {
    Root {
        discovery: String,
    },
    Delegated {
        discovery: String,
        transition: String,
        parent: Box<WorthQueryDurableAuthorizationDependencies>,
    },
}

impl WorthQueryAuthorizationDecisionFact {
    pub(in crate::domain_computation) fn retained_durable_dependencies(
        &self,
    ) -> Result<WorthQueryDurableAuthorizationDependencies, ()> {
        Ok(WorthQueryDurableAuthorizationDependencies {
            primary: self.native_dependencies.clone().ok_or(())?,
            delegation: self
                .delegation
                .as_ref()
                .map(|delegation| delegation.retained_durable_dependencies())
                .transpose()?,
            activation: self
                .delegation_activation
                .as_ref()
                .map(|activation| activation.retained_durable_dependencies())
                .transpose()?,
        })
    }
}

impl WorthQueryDurableAuthorizationDependencies {
    pub(in crate::domain_computation) fn validate(&self) -> Result<(), ()> {
        validate(&self.primary)?;
        if let Some(delegation) = &self.delegation {
            delegation.validate()?;
        }
        if let Some(activation) = &self.activation {
            validate(activation)?;
        }
        Ok(())
    }
}

impl WorthQueryDurableDelegationDependencies {
    fn validate(&self) -> Result<(), ()> {
        match self {
            Self::Root { discovery } => validate(discovery),
            Self::Delegated {
                discovery,
                transition,
                parent,
            } => {
                validate(discovery)?;
                validate(transition)?;
                parent.validate()
            }
        }
    }
}

pub(super) fn capture(
    runtime: &RelationalRuntime,
    evidence: &RelationalAuthorizationObservationEvidence,
) -> Result<String, ()> {
    runtime
        .capture_authorization_durable_dependencies(evidence)
        .map_err(|_| ())?
        .to_wire_string()
        .map_err(|_| ())
}

fn validate(encoded: &str) -> Result<(), ()> {
    RelationalAuthorizationDurableDependencies::from_wire_string(encoded)
        .map(|_| ())
        .map_err(|_| ())
}
