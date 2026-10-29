#[path = "contested.rs"]
pub(crate) mod contested;

#[path = "empty.rs"]
pub(crate) mod empty;

#[path = "operating.rs"]
pub(crate) mod operating;

#[path = "retention_pressure.rs"]
pub(crate) mod retention_pressure;

#[path = "version_boundary.rs"]
pub(crate) mod version_boundary;

pub(crate) use version_boundary::hazard_v2_transition;

pub(crate) use super::definition::SupplyChainWorldDefinition;
pub(crate) use super::oracle::OracleBranch;
pub(crate) use super::scale::SupplyChainScale;
pub(crate) use super::schema::SchemaVersion;
pub(crate) use super::semantic_key::BranchLabel;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BaselineName {
    EmptyInstallation,
    Operating,
    ContestedPlanning,
    RetentionPressure,
    VersionBoundary,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum RetentionObligationKind {
    Snapshot,
    Observation,
    Transaction,
    Candidate,
    ExternalBasis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetentionObligation {
    pub(crate) target: BranchLabel,
    pub(crate) ancestor_path: Vec<BranchLabel>,
    pub(crate) kind: RetentionObligationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetentionObligationError {
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
pub(crate) struct BranchCreationIntent {
    pub(crate) branch: BranchLabel,
    pub(crate) parent: BranchLabel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BranchIntentError {
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
pub(crate) struct SupplyChainBaseline {
    pub(crate) name: BaselineName,
    pub(crate) scale: SupplyChainScale,
    pub(crate) definition: SupplyChainWorldDefinition,
    pub(crate) branch: OracleBranch,
    pub(crate) branch_intents: Vec<BranchCreationIntent>,
    pub(crate) retention_obligations: Vec<RetentionObligation>,
    pub(crate) pre_upgrade_schema: Option<SchemaVersion>,
    pub(crate) post_upgrade_schema: Option<SchemaVersion>,
}

impl SupplyChainBaseline {
    pub(crate) fn empty(scale: SupplyChainScale) -> Self {
        empty::build(scale)
    }

    pub(crate) fn operating(scale: SupplyChainScale) -> Self {
        operating::build(scale)
    }

    pub(crate) fn contested(scale: SupplyChainScale) -> Self {
        contested::build(scale)
    }

    pub(crate) fn retention_pressure(scale: SupplyChainScale) -> Self {
        retention_pressure::build(scale)
    }

    pub(crate) fn version_boundary(scale: SupplyChainScale) -> Self {
        version_boundary::build(scale)
    }

    pub(crate) fn validate_branch_intents(&self) -> Result<(), BranchIntentError> {
        let current = self.branch.ancestry.branch;
        let mut pairs = std::collections::BTreeSet::new();
        let mut children = std::collections::BTreeSet::new();
        for intent in &self.branch_intents {
            if intent.branch == intent.parent {
                return Err(BranchIntentError::ChildEqualsParent(intent.branch));
            }
            if intent.branch == current {
                return Err(BranchIntentError::CurrentBranch(intent.branch));
            }
            if !pairs.insert((intent.branch, intent.parent)) {
                return Err(BranchIntentError::DuplicatePair(
                    intent.branch,
                    intent.parent,
                ));
            }
            if !children.insert(intent.branch) {
                return Err(BranchIntentError::DuplicateChild(intent.branch));
            }
            if intent.parent != current {
                return Err(BranchIntentError::ParentUnavailable {
                    branch: intent.branch,
                    parent: intent.parent,
                });
            }
            self.branch
                .fork(intent.branch, intent.parent)
                .map_err(|_| BranchIntentError::CurrentBranch(intent.branch))?;
        }
        Ok(())
    }

    pub(crate) fn validate_retention_obligations(&self) -> Result<(), RetentionObligationError> {
        let mut kinds = std::collections::BTreeSet::new();
        for obligation in &self.retention_obligations {
            let Some(root) = obligation.ancestor_path.first().copied() else {
                return Err(RetentionObligationError::EmptyPath(obligation.kind));
            };
            if root != BranchLabel::Operating {
                return Err(RetentionObligationError::WrongRoot {
                    kind: obligation.kind,
                    observed: root,
                });
            }
            if !self
                .branch_intents
                .iter()
                .any(|intent| intent.branch == obligation.target)
            {
                return Err(RetentionObligationError::UnknownTarget {
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
                    return Err(RetentionObligationError::UnknownAncestorPath {
                        kind: obligation.kind,
                        parent: *parent,
                        child: *child,
                    });
                }
            }
            if !kinds.insert(obligation.kind) {
                return Err(RetentionObligationError::DuplicateKind(obligation.kind));
            }
        }
        Ok(())
    }
}
