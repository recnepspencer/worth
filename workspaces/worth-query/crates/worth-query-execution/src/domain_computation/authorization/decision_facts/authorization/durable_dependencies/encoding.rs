//! Borrowed native dependency descriptions; only recursive view nodes allocate.
use super::*;
use crate::domain_computation::authorization::WorkflowApprovalRequestEncodingDenial as Denial;
use crate::domain_computation::primary_graph::workflow::actor::{
    encoding::FundedActorText, fund_actor_codec, ActorCodecFunding,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

#[derive(Serialize)]
pub(in crate::domain_computation) struct DurableDependenciesView<'a> {
    primary: &'a str,
    recorded_actor: RecordedActor,
    delegation: Option<Delegation<'a>>,
    activation: Option<&'a str>,
}

// Descriptive comparator input from the fixture's genuine native capture. This
// neither mints capability metadata nor changes the conventional decision.
#[cfg(test)]
pub(super) fn conventional_observed_view(
    primary: &worth_relational::facade::authorization::RelationalAuthorizationDependencyWire,
) -> DurableDependenciesView<'_> {
    DurableDependenciesView {
        primary: primary.as_str(),
        recorded_actor: RecordedActor::NotApplicable,
        delegation: None,
        activation: None,
    }
}

#[derive(Serialize)]
enum RecordedActor {
    NotApplicable,
    Native(FundedActorText),
}

#[derive(Serialize)]
enum Delegation<'a> {
    Root {
        discovery: &'a str,
    },
    Delegated {
        discovery: &'a str,
        transition: &'a str,
        parent: Box<DurableDependenciesView<'a>>,
        #[serde(skip)]
        _funding: Arc<ActorCodecFunding>,
    },
}

pub(super) fn capture<'a>(
    fact: &'a WorthQueryAuthorizationDecisionFact,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<DurableDependenciesView<'a>, Denial> {
    use super::super::delegation::WorthQueryDelegationDecisionFact as Native;
    crate::domain_computation::primary_graph::workflow::actor::encoding::check_live(execution)
        .map_err(|cause| Denial::Native(Box::new(cause)))?;
    let primary = fact
        .native_dependencies
        .as_deref()
        .ok_or(Denial::InvalidRequest)?;
    let delegation = fact
        .delegation
        .as_ref()
        .map(|delegation| -> Result<_, Denial> {
            Ok(match delegation.as_ref() {
                Native::Root {
                    discovery_dependencies,
                    ..
                } => Delegation::Root {
                    discovery: discovery_dependencies,
                },
                Native::Delegated {
                    discovery_dependencies,
                    transition_dependencies,
                    parent,
                    ..
                } => {
                    let funding = fund_actor_codec(
                        execution,
                        std::mem::size_of::<DurableDependenciesView<'_>>(),
                    )
                    .map_err(|cause| Denial::Native(Box::new(cause)))?;
                    Delegation::Delegated {
                        discovery: discovery_dependencies,
                        transition: transition_dependencies,
                        parent: Box::new(capture(parent, execution)?),
                        _funding: funding,
                    }
                }
            })
        })
        .transpose()?;
    Ok(DurableDependenciesView {
        primary,
        recorded_actor: match &fact.recorded_actor {
            None => RecordedActor::NotApplicable,
            Some(actor) => RecordedActor::Native(actor.encode(execution, crate::domain_computation::authorization::workflow_approval_dependencies::MAXIMUM_APPROVAL_DEPENDENCY_BYTES)?),
        },
        delegation,
        activation: fact
            .delegation_activation
            .as_ref()
            .map(|activation| activation.retained_durable_dependencies()),
    })
}

impl WorthQueryDurableAuthorizationDependencies {
    pub(in crate::domain_computation) fn matches_view(
        &self,
        view: &DurableDependenciesView<'_>,
    ) -> bool {
        self.activation.as_deref() == view.activation && self.matches_view_without_activation(view)
    }

    pub(super) fn matches_view_without_activation(
        &self,
        view: &DurableDependenciesView<'_>,
    ) -> bool {
        self.primary == view.primary
            && match (&self.recorded_actor, &view.recorded_actor) {
                (DurableRecordedActor::NotApplicable, RecordedActor::NotApplicable) => true,
                (DurableRecordedActor::Native(left), RecordedActor::Native(right)) => {
                    left == right.as_str()
                }
                _ => false,
            }
            && match (&self.delegation, &view.delegation) {
                (None, None) => true,
                (
                    Some(WorthQueryDurableDelegationDependencies::Root { discovery: left }),
                    Some(Delegation::Root { discovery: right }),
                ) => left.as_str() == *right,
                (
                    Some(WorthQueryDurableDelegationDependencies::Delegated {
                        discovery: ld,
                        transition: lt,
                        parent: lp,
                    }),
                    Some(Delegation::Delegated {
                        discovery: rd,
                        transition: rt,
                        parent: rp,
                        ..
                    }),
                ) => ld.as_str() == *rd && lt.as_str() == *rt && lp.matches_view(rp),
                _ => false,
            }
    }
}

impl DurableDependenciesView<'_> {
    pub(super) fn has_activation(&self) -> bool {
        self.activation.is_some()
    }
}
