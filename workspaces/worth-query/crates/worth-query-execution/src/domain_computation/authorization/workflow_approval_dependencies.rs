//! Bounded, descriptive source-native dependencies retained with one approval.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
mod parse_inventory;
use super::WorkflowApprovalRequestDecodingDenial as DecodeDenial;
use crate::domain_computation::primary_graph::workflow::actor::{
    fund_actor_codec, ActorCodecFunding,
};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

use super::{
    WorkflowApprovalRequestEncodingDenial, WorthQueryDurableAuthorizationDependencies,
    WorthQueryRetainedCapabilityAuthorization,
};

const VERSION: u8 = 2;
// Each nested Relational stamp may use its full bounded wire budget; this
// envelope has its own aggregate limit rather than the request-text limit.
pub(super) const MAXIMUM_APPROVAL_DEPENDENCY_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dependencies {
    version: u8,
    primary: WorthQueryDurableAuthorizationDependencies,
    support: Option<WorthQueryDurableAuthorizationDependencies>,
}

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryWorkflowApprovalDependencies {
    owned: Arc<OwnedDependencies>,
}
struct OwnedDependencies {
    value: Dependencies,
    _text_funding: Arc<ActorCodecFunding>,
    _node_funding: Arc<ActorCodecFunding>,
}
pub(in crate::domain_computation) fn encode_workflow_approval_dependencies(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authorization: &WorthQueryRetainedCapabilityAuthorization,
) -> Result<
    crate::domain_computation::primary_graph::workflow::actor::encoding::FundedActorText,
    WorkflowApprovalRequestEncodingDenial,
> {
    #[derive(Serialize)]
    struct Dependencies<P, S> {
        version: u8,
        primary: P,
        support: S,
    }
    let dependencies = Dependencies {
        version: VERSION,
        primary: authorization
            .decision()
            .retained_durable_dependencies(execution)?,
        support: authorization
            .supporting()
            .map(|support| support.decision().retained_durable_dependencies(execution))
            .transpose()?,
    };
    crate::domain_computation::primary_graph::workflow::actor::encoding::encode(
        execution,
        &dependencies,
        MAXIMUM_APPROVAL_DEPENDENCY_BYTES,
    )
}

impl WorthQueryWorkflowApprovalDependencies {
    pub(in crate::domain_computation) fn decode(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        encoded: &str,
    ) -> Result<Self, DecodeDenial> {
        if encoded.len() > MAXIMUM_APPROVAL_DEPENDENCY_BYTES {
            return Err(().into());
        }
        // Escaped string scratch <2*input; owned decoded text <=input.
        let text_funding = fund_actor_codec(execution, encoded.len().checked_mul(3).ok_or(())?)?;
        let inventory: parse_inventory::Inventory =
            serde_json::from_str(encoded).map_err(|_| ())?;
        if inventory.version != VERSION {
            return Err(().into());
        }
        let node_funding = fund_actor_codec(
            execution,
            inventory
                .boxed_nodes()
                .checked_mul(std::mem::size_of::<
                    WorthQueryDurableAuthorizationDependencies,
                >())
                .ok_or(())?,
        )?;
        let dependencies: Dependencies = serde_json::from_str(encoded).map_err(|_| ())?;
        crate::domain_computation::primary_graph::workflow::actor::encoding::check_live(execution)?;
        if dependencies.version != VERSION {
            return Err(().into());
        }
        dependencies.primary.validate(execution)?;
        if let Some(support) = &dependencies.support {
            support.validate(execution)?;
        }
        Ok(Self {
            owned: Arc::new(OwnedDependencies {
                value: dependencies,
                _text_funding: text_funding,
                _node_funding: node_funding,
            }),
        })
    }

    pub(in crate::domain_computation) fn support_present(&self) -> bool {
        self.owned.value.support.is_some()
    }

    pub(in crate::domain_computation) fn primary_matches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &super::WorthQueryAuthorizationDecisionFact,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenial> {
        match current.retained_durable_dependencies(execution) {
            Ok(observed) => Ok(self.owned.value.primary.matches_view(&observed)),
            Err(WorkflowApprovalRequestEncodingDenial::Native(cause)) => {
                Err(super::WorthQueryOperationAuthorizationDenial::workflow_actor_native(*cause))
            }
            Err(_) => Ok(false),
        }
    }

    pub(in crate::domain_computation) fn support_matches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &super::WorthQueryAuthorizationDecisionFact,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenial> {
        let Some(expected) = &self.owned.value.support else {
            return Ok(false);
        };
        match current.retained_durable_dependencies(execution) {
            Ok(observed) => Ok(expected.matches_view(&observed)),
            Err(WorkflowApprovalRequestEncodingDenial::Native(cause)) => {
                Err(super::WorthQueryOperationAuthorizationDenial::workflow_actor_native(*cause))
            }
            Err(_) => Ok(false),
        }
    }

    pub(in crate::domain_computation) fn delegation_target_support_matches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &super::WorthQueryAuthorizationDecisionFact,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, super::WorthQueryOperationAuthorizationDenial> {
        execution.check_live().map_err(|stop| {
            super::WorthQueryOperationAuthorizationDenial::new(
                super::denial::kernel_stop_kind(stop),
                "workflow approval dependencies",
            )
        })?;
        let Some(expected) = &self.owned.value.support else {
            return Ok(false);
        };
        let observed = match current.retained_durable_dependencies(execution) {
            Ok(observed) => observed,
            Err(WorkflowApprovalRequestEncodingDenial::Native(cause)) => {
                return Err(
                    super::WorthQueryOperationAuthorizationDenial::workflow_actor_native(*cause),
                )
            }
            Err(_) => return Ok(false),
        };
        expected.matches_with_retained_activation(execution, &observed, runtime, snapshot)
    }
}
