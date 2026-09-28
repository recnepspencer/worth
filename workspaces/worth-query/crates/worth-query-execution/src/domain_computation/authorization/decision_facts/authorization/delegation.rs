use std::sync::Arc;

use serde::{Deserialize, Serialize};
use worth_relational::facade::authorization::{
    RelationalAuthorizationObservationCounters, RelationalAuthorizationObservationEvidence,
};
use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;

use super::{add_counters, observation_is_current, WorthQueryAuthorizationDecisionFact};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::domain_computation) enum WorthQueryDurableCapabilityLineage {
    Unbound,
    Root {
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    },
    Delegated {
        grantor: worth_relational::facade::identity::EntityId,
        parent_grant: worth_relational::facade::identity::EntityId,
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
        parent: Box<WorthQueryDurableCapabilityLineage>,
    },
}

#[derive(Clone)]
pub(in crate::domain_computation::authorization) enum WorthQueryDelegationDecisionFact {
    Root {
        discovery: Arc<RelationalAuthorizationObservationEvidence>,
        discovery_dependencies: String,
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    },
    Delegated {
        grantor: worth_relational::facade::identity::EntityId,
        parent_grant: worth_relational::facade::identity::EntityId,
        discovery: Arc<RelationalAuthorizationObservationEvidence>,
        discovery_dependencies: String,
        transition: Arc<RelationalAuthorizationObservationEvidence>,
        transition_dependencies: String,
        parent: Arc<WorthQueryAuthorizationDecisionFact>,
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    },
}

impl WorthQueryDelegationDecisionFact {
    pub(super) fn retained_durable_dependencies(
        &self,
    ) -> Result<super::durable_dependencies::WorthQueryDurableDelegationDependencies, ()> {
        use super::durable_dependencies::WorthQueryDurableDelegationDependencies;
        match self {
            Self::Root {
                discovery_dependencies,
                ..
            } => Ok(WorthQueryDurableDelegationDependencies::Root {
                discovery: discovery_dependencies.clone(),
            }),
            Self::Delegated {
                discovery_dependencies,
                transition_dependencies,
                parent,
                ..
            } => Ok(WorthQueryDurableDelegationDependencies::Delegated {
                discovery: discovery_dependencies.clone(),
                transition: transition_dependencies.clone(),
                parent: Box::new(parent.retained_durable_dependencies()?),
            }),
        }
    }

    pub(super) fn durable_lineage(&self) -> WorthQueryDurableCapabilityLineage {
        match self {
            Self::Root {
                status_revision, ..
            } => WorthQueryDurableCapabilityLineage::Root {
                status_revision: *status_revision,
            },
            Self::Delegated {
                grantor,
                parent_grant,
                parent,
                status_revision,
                ..
            } => WorthQueryDurableCapabilityLineage::Delegated {
                grantor: *grantor,
                parent_grant: *parent_grant,
                status_revision: *status_revision,
                parent: Box::new(parent.durable_lineage()),
            },
        }
    }
    pub(in crate::domain_computation::authorization) fn root(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        discovery: RelationalAuthorizationObservationEvidence,
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    ) -> Result<Self, ()> {
        let discovery_dependencies = super::durable_dependencies::capture(runtime, &discovery)?;
        Ok(Self::Root {
            discovery: Arc::new(discovery),
            discovery_dependencies,
            status_revision,
        })
    }

    pub(in crate::domain_computation::authorization) fn delegated(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        grantor: worth_relational::facade::identity::EntityId,
        parent_grant: worth_relational::facade::identity::EntityId,
        discovery: RelationalAuthorizationObservationEvidence,
        transition: RelationalAuthorizationObservationEvidence,
        parent: WorthQueryAuthorizationDecisionFact,
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    ) -> Result<Self, ()> {
        let discovery_dependencies = super::durable_dependencies::capture(runtime, &discovery)?;
        let transition_dependencies = super::durable_dependencies::capture(runtime, &transition)?;
        Ok(Self::Delegated {
            grantor,
            parent_grant,
            discovery: Arc::new(discovery),
            discovery_dependencies,
            transition: Arc::new(transition),
            transition_dependencies,
            parent: Arc::new(parent),
            status_revision,
        })
    }

    pub(super) fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        match self {
            Self::Root { .. } => true,
            Self::Delegated { parent, .. } => parent.session_identity() == session,
        }
    }

    pub(super) fn add_relational_counters(
        &self,
        counters: &mut RelationalAuthorizationObservationCounters,
    ) {
        match self {
            Self::Root { discovery, .. } => add_counters(counters, discovery.counters()),
            Self::Delegated {
                discovery,
                transition,
                parent,
                ..
            } => {
                add_counters(counters, discovery.counters());
                add_counters(counters, transition.counters());
                add_counters(counters, parent.relational_counters());
            }
        }
    }

    pub(super) fn signal_dependency_count(&self) -> usize {
        match self {
            Self::Root { .. } => 0,
            Self::Delegated { parent, .. } => parent.signal_dependency_count(),
        }
    }

    pub(super) fn bridge_is_retained(&self, bridge: &BridgeAuthorizationRuntime) -> bool {
        match self {
            Self::Root { .. } => true,
            Self::Delegated { parent, .. } => parent.bridge_is_retained(bridge),
        }
    }

    pub(super) fn remains_current_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        bridge: &BridgeAuthorizationRuntime,
    ) -> bool {
        self.remains_equal_in(runtime, snapshot)
            && match self {
                Self::Root { .. } => true,
                Self::Delegated { parent, .. } => {
                    parent.remains_current_in(runtime, snapshot, bridge)
                }
            }
    }

    pub(super) fn remains_equal_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        match self {
            Self::Root { discovery, .. } => observation_is_current(runtime, snapshot, discovery),
            Self::Delegated {
                discovery,
                transition,
                parent,
                ..
            } => {
                observation_is_current(runtime, snapshot, discovery)
                    && observation_is_current(runtime, snapshot, transition)
                    && parent.remains_equal_in(runtime, snapshot)
            }
        }
    }

    pub(super) fn has_same_lineage(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Root {
                    status_revision: left,
                    ..
                },
                Self::Root {
                    status_revision: right,
                    ..
                },
            ) => left == right,
            (
                Self::Delegated {
                    grantor: left_grantor,
                    parent_grant: left_parent,
                    parent: left,
                    status_revision: left_revision,
                    ..
                },
                Self::Delegated {
                    grantor: right_grantor,
                    parent_grant: right_parent,
                    parent: right,
                    status_revision: right_revision,
                    ..
                },
            ) => {
                left_grantor == right_grantor
                    && left_parent == right_parent
                    && left_revision == right_revision
                    && left.has_same_lineage(right)
            }
            (Self::Root { .. }, Self::Delegated { .. })
            | (Self::Delegated { .. }, Self::Root { .. }) => false,
        }
    }
}
