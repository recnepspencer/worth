pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenarios::version_boundary::hazard_v2_transition};

pub use crate::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition;
pub use crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch;
pub use crate::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale;
pub use crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion;
pub use crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BaselineName {
    EmptyInstallation,
    Operating,
    ContestedPlanning,
    RetentionPressure,
    VersionBoundary,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RetentionObligationKind {
    Snapshot,
    Observation,
    Transaction,
    Candidate,
    ExternalBasis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionObligation {
    pub target: BranchLabel,
    pub ancestor_path: Vec<BranchLabel>,
    pub kind: RetentionObligationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionObligationError {
    EmptyPath(RetentionObligationKind),
    WrongRoot {
        kind: RetentionObligationKind,
        observed: BranchLabel,
    },
    UnknownAncestorPath {
        kind: RetentionObligationKind,
        parent: BranchLabel,
        child: BranchLabel,
    },
    UnknownTarget {
        kind: RetentionObligationKind,
        target: BranchLabel,
    },
    DuplicateKind(RetentionObligationKind),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BranchCreationIntent {
    pub branch: BranchLabel,
    pub parent: BranchLabel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchIntentError {
    ChildEqualsParent(BranchLabel),
    CurrentBranch(BranchLabel),
    ParentUnavailable {
        branch: BranchLabel,
        parent: BranchLabel,
    },
    DuplicatePair(BranchLabel, BranchLabel),
    DuplicateChild(BranchLabel),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainBaseline {
    pub name: BaselineName,
    pub scale: SupplyChainScale,
    pub definition: SupplyChainWorldDefinition,
    pub branch: OracleBranch,
    pub branch_intents: Vec<BranchCreationIntent>,
    pub retention_obligations: Vec<RetentionObligation>,
    pub pre_upgrade_schema: Option<SchemaVersion>,
    pub post_upgrade_schema: Option<SchemaVersion>,
}

impl SupplyChainBaseline {
    pub fn empty(scale: SupplyChainScale) -> Self {
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::empty::build(scale)
    }

    pub fn operating(scale: SupplyChainScale) -> Self {
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::operating::build(scale)
    }

    pub fn contested(scale: SupplyChainScale) -> Self {
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::contested::build(scale)
    }

    pub fn retention_pressure(scale: SupplyChainScale) -> Self {
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::retention_pressure::build(scale)
    }

    pub fn version_boundary(scale: SupplyChainScale) -> Self {
        crate::relational_certification::ordinary_context::world::supply_chain::scenarios::version_boundary::build(scale)
    }

    pub fn validate_branch_intents(&self) -> Result<(), BranchIntentError> {
        let current = self.branch.ancestry.branch;
        let mut pairs = std::collections::BTreeSet::new();
        let mut children = std::collections::BTreeSet::new();
        for intent in &self.branch_intents {
            if intent.branch == intent.parent {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::ChildEqualsParent(intent.branch));
            }
            if intent.branch == current {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::CurrentBranch(intent.branch));
            }
            if !pairs.insert((intent.branch, intent.parent)) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::DuplicatePair(
                    intent.branch,
                    intent.parent,
                ));
            }
            if !children.insert(intent.branch) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::DuplicateChild(intent.branch));
            }
            if intent.parent != current {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::ParentUnavailable {
                    branch: intent.branch,
                    parent: intent.parent,
                });
            }
            self.branch
                .fork(intent.branch, intent.parent)
                .map_err(|_| crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::CurrentBranch(intent.branch))?;
        }
        Ok(())
    }

    pub fn validate_retention_obligations(&self) -> Result<(), RetentionObligationError> {
        let mut kinds = std::collections::BTreeSet::new();
        for obligation in &self.retention_obligations {
            let Some(root) = obligation.ancestor_path.first().copied() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::EmptyPath(obligation.kind));
            };
            if root != crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::WrongRoot {
                    kind: obligation.kind,
                    observed: root,
                });
            }
            if !self
                .branch_intents
                .iter()
                .any(|intent| intent.branch == obligation.target)
            {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::UnknownTarget {
                    kind: obligation.kind,
                    target: obligation.target,
                });
            }
            for pair in obligation.ancestor_path.windows(2) {
                let [parent, child] = pair else {
                    continue;
                };
                if !self
                    .branch_intents
                    .iter()
                    .any(|intent| intent.parent == *parent && intent.branch == *child)
                {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::UnknownAncestorPath {
                        kind: obligation.kind,
                        parent: *parent,
                        child: *child,
                    });
                }
            }
            if !kinds.insert(obligation.kind) {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::DuplicateKind(obligation.kind));
            }
        }
        Ok(())
    }
}
