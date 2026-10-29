pub(crate) use std::collections::BTreeSet;

pub(crate) use super::super::delta::DeltaId;
pub(crate) use super::super::semantic_key::BranchLabel;
pub(crate) use super::state::OracleState;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct AcceptedDelta {
    pub(crate) branch: BranchLabel,
    pub(crate) delta: DeltaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OracleAncestry {
    pub(crate) branch: BranchLabel,
    pub(crate) parent: Option<BranchLabel>,
    pub(crate) lineage: Vec<BranchLabel>,
    pub(crate) accepted: Vec<DeltaId>,
    pub(crate) history: Vec<AcceptedDelta>,
}

impl OracleAncestry {
    pub(crate) fn common_ancestor(&self, other: &Self) -> Option<BranchLabel> {
        self.lineage
            .iter()
            .zip(&other.lineage)
            .take_while(|(left, right)| left == right)
            .map(|(branch, _)| *branch)
            .last()
    }

    /// Records an ordered semantic history without claiming a domain replay.
    /// The production oracle uses `OracleBranch::accept` for applied deltas;
    /// this fixture is only for ancestry-order proofs.
    pub(crate) fn record_history(&self, history: &[AcceptedDelta]) -> Result<Self, AncestryError> {
        if let Some(event) = history
            .iter()
            .find(|event| !self.lineage.contains(&event.branch))
        {
            return Err(AncestryError::HistoryOwnerUnavailable(event.branch));
        }
        let mut next = self.clone();
        next.accepted
            .extend(history.iter().map(|event| event.delta));
        next.history.extend_from_slice(history);
        Ok(next)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OracleBranch {
    pub(crate) state: OracleState,
    pub(crate) ancestry: OracleAncestry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AncestryError {
    BranchAlreadyExists(BranchLabel),
    ParentMismatch {
        expected: BranchLabel,
        observed: BranchLabel,
    },
    LineageMismatch {
        expected: Vec<BranchLabel>,
        observed: Vec<BranchLabel>,
    },
    HistoryOwnerUnavailable(BranchLabel),
    WrongBranch(BranchLabel),
}

impl OracleBranch {
    pub(crate) fn genesis(state: OracleState) -> Self {
        Self {
            state,
            ancestry: OracleAncestry {
                branch: BranchLabel::Operating,
                parent: None,
                lineage: vec![BranchLabel::Operating],
                accepted: Vec::new(),
                history: Vec::new(),
            },
        }
    }

    pub(crate) fn fork(
        &self,
        branch: BranchLabel,
        parent: BranchLabel,
    ) -> Result<Self, AncestryError> {
        if self.ancestry.lineage.contains(&branch) {
            return Err(AncestryError::BranchAlreadyExists(branch));
        }
        if parent != self.ancestry.branch {
            return Err(AncestryError::ParentMismatch {
                expected: self.ancestry.branch,
                observed: parent,
            });
        }
        Ok(Self {
            state: self.state.clone(),
            ancestry: OracleAncestry {
                branch,
                parent: Some(parent),
                lineage: self
                    .ancestry
                    .lineage
                    .iter()
                    .copied()
                    .chain([branch])
                    .collect(),
                accepted: Vec::new(),
                history: self.ancestry.history.clone(),
            },
        })
    }

    pub(crate) fn accept(&self, delta: DeltaId) -> Self {
        let mut next = self.clone();
        next.ancestry.accepted.push(delta);
        next.ancestry.history.push(AcceptedDelta {
            branch: next.ancestry.branch,
            delta,
        });
        next
    }

    pub(crate) fn expects_parent(&self, branch: BranchLabel) -> Result<(), AncestryError> {
        match self.ancestry.parent {
            Some(parent) if parent == branch => Ok(()),
            Some(expected) => Err(AncestryError::ParentMismatch {
                expected,
                observed: branch,
            }),
            None => Err(AncestryError::WrongBranch(branch)),
        }
    }

    pub(crate) fn expects_parent_ancestry(
        &self,
        parent: &OracleAncestry,
    ) -> Result<(), AncestryError> {
        self.expects_parent(parent.branch)?;
        let expected = parent
            .lineage
            .iter()
            .copied()
            .chain([self.ancestry.branch])
            .collect::<Vec<_>>();
        if !valid_lineage(parent.branch, &parent.lineage)
            || !valid_lineage(self.ancestry.branch, &self.ancestry.lineage)
            || self.ancestry.lineage != expected
        {
            return Err(AncestryError::LineageMismatch {
                expected,
                observed: self.ancestry.lineage.clone(),
            });
        }
        Ok(())
    }
}

pub(crate) fn valid_lineage(branch: BranchLabel, lineage: &[BranchLabel]) -> bool {
    lineage.first() == Some(&BranchLabel::Operating)
        && lineage.last() == Some(&branch)
        && lineage.iter().copied().collect::<BTreeSet<_>>().len() == lineage.len()
}
