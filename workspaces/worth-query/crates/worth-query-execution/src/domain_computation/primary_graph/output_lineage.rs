//! Platform-owned semantic output correspondence at committed graph bases.

use std::collections::BTreeMap;

use worth_relational::facade::{history::CommitId, identity::EntityId};

use super::{
    provider::WorthQueryPrimaryGraphCommittedApplication, WorthQueryApplicationOutputAction,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputProjectionDenial,
    WorthQueryApplicationOutputRole,
};

struct CommittedOutputLineage {
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    correspondence: WorthQueryApplicationOutputCorrespondence,
}

#[derive(Default)]
pub(super) struct WorthQueryApplicationOutputLineage {
    by_commit: BTreeMap<CommitId, CommittedOutputLineage>,
}

pub(super) struct WorthQueryPriorOutputResolution {
    pub(super) entity: EntityId,
    pub(super) commit_probes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPriorOutputDenialKind {
    Unavailable,
    MissingRole,
    ActionMismatch,
    EntityMismatch,
    OutputUnavailable,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPriorOutputDenial {
    kind: WorthQueryPriorOutputDenialKind,
    role: String,
}

impl WorthQueryApplicationOutputLineage {
    pub(super) fn record(&mut self, application: &WorthQueryPrimaryGraphCommittedApplication) {
        let evidence = application.commit_evidence();
        let previous = self.by_commit.insert(
            evidence.commit_reference().commit_id,
            CommittedOutputLineage {
                scope: evidence.operation_scope().clone(),
                correspondence: evidence.output_correspondence().clone(),
            },
        );
        assert!(
            previous.is_none(),
            "one commit may seal output lineage only once"
        );
    }

    pub(super) fn resolve<Binding, Entity, Action>(
        &self,
        ancestry: &[CommitId],
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
    ) -> Result<WorthQueryPriorOutputResolution, WorthQueryPriorOutputDenial>
    where
        Binding: 'static,
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        let mut commit_probes = 0;
        for commit in ancestry.iter().rev() {
            commit_probes += 1;
            let Some(lineage) = self.by_commit.get(commit) else {
                continue;
            };
            if !same_scope(&lineage.scope, scope) || !lineage.correspondence.belongs_to::<Binding>()
            {
                continue;
            }
            return lineage
                .correspondence
                .entity(role)
                .map(|entity| WorthQueryPriorOutputResolution {
                    entity: entity.entity_id(),
                    commit_probes,
                })
                .map_err(|denial| WorthQueryPriorOutputDenial::projection(role.name(), denial));
        }
        Err(WorthQueryPriorOutputDenial::new(
            WorthQueryPriorOutputDenialKind::MissingRole,
            role.name(),
        ))
    }
}

fn same_scope(
    left: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    right: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
) -> bool {
    left.runtime_authority() == right.runtime_authority()
        && left.binding_identity() == right.binding_identity()
        && left.principal() == right.principal()
        && left.scope() == right.scope()
}

impl WorthQueryPriorOutputDenial {
    pub const fn kind(&self) -> WorthQueryPriorOutputDenialKind {
        self.kind
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub(super) fn new(kind: WorthQueryPriorOutputDenialKind, role: impl Into<String>) -> Self {
        Self {
            kind,
            role: role.into(),
        }
    }

    fn projection(role: &str, denial: WorthQueryApplicationOutputProjectionDenial) -> Self {
        let kind = match denial {
            WorthQueryApplicationOutputProjectionDenial::MissingRole => {
                WorthQueryPriorOutputDenialKind::MissingRole
            }
            WorthQueryApplicationOutputProjectionDenial::ForeignBinding => {
                WorthQueryPriorOutputDenialKind::Unavailable
            }
            WorthQueryApplicationOutputProjectionDenial::ActionMismatch => {
                WorthQueryPriorOutputDenialKind::ActionMismatch
            }
            WorthQueryApplicationOutputProjectionDenial::EntityMismatch => {
                WorthQueryPriorOutputDenialKind::EntityMismatch
            }
        };
        Self::new(kind, role)
    }
}

impl std::fmt::Display for WorthQueryPriorOutputDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "prior output denied: {:?} ({})",
            self.kind, self.role
        )
    }
}

impl std::error::Error for WorthQueryPriorOutputDenial {}
