//! An installed executable admits an actual structural cut, never a complete boolean.
use std::sync::Arc;
mod frame;
pub use frame::BorrowedManagedCoverageFrame;

#[derive(Debug)]
pub struct ManagedIndexCoverageIssuer(Arc<()>);
#[derive(Clone, Debug)]
pub struct ManagedIndexCoverageAuthority(Arc<()>);
#[derive(Debug)]
/// Coverage cannot be assembled from a caller's descriptive facts.
/// ```compile_fail
/// let proof = worth_proof::AdmittedManagedIndexCoverage {
///     owner: std::sync::Arc::new(()), matched: 0,
/// };
/// ```
pub struct AdmittedManagedIndexCoverage {
    owner: Arc<()>,
    matched: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedCoverageDisposition {
    Branch,
    Matched,
    Excluded,
    Pruned,
}
impl ManagedCoverageDisposition {
    pub const fn expanded(self) -> bool {
        !matches!(self, Self::Pruned)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagedCoverageStep {
    pub node: usize,
    pub disposition: ManagedCoverageDisposition,
}
/// Descriptive facts borrowed from the owner's already checked Source tree.
#[derive(Clone, Copy, Debug)]
pub struct ManagedCoverageRegion {
    pub parent: Option<usize>,
    pub children: [Option<usize>; 2],
    pub has_source: bool,
}
pub struct ManagedCoverageFrontier<'a> {
    pub root: Option<usize>,
    pub recorded_regions: usize,
    pub allow_pruning: bool,
    pub steps: &'a mut dyn Iterator<Item = ManagedCoverageStep>,
    pub region: &'a mut dyn FnMut(usize) -> Option<ManagedCoverageRegion>,
    pub disposition: &'a mut dyn FnMut(usize) -> Option<ManagedCoverageDisposition>,
    pub checkpoint: &'a mut dyn FnMut() -> bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedIndexCoverageDenial {
    Interrupted,
    MissingRoot,
    UnorderedRegion,
    MissingRegion,
    InvalidParent,
    MissingChild,
    SourceDisposition,
    PruningUnavailable,
    CountOverflow,
    IncompleteFrontier,
}
impl ManagedIndexCoverageIssuer {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
    pub fn authority(&self) -> ManagedIndexCoverageAuthority {
        ManagedIndexCoverageAuthority(Arc::clone(&self.0))
    }
    /// Only the actual executable's verifier keeps its matching issuer. This
    /// structural cut still requires that owner's invocation and premise checks.
    pub fn verify(
        &self,
        frontier: ManagedCoverageFrontier<'_>,
    ) -> Result<AdmittedManagedIndexCoverage, ManagedIndexCoverageDenial> {
        use ManagedIndexCoverageDenial as D;
        if !(frontier.checkpoint)() {
            return Err(D::Interrupted);
        }
        if let Some(root) = frontier.root {
            if (frontier.disposition)(root).is_none() {
                return Err(D::MissingRoot);
            }
        }
        let mut previous = None;
        let mut matched = 0usize;
        let mut recorded = 0usize;
        let mut root_seen = frontier.root.is_none();
        for step in frontier.steps {
            if !(frontier.checkpoint)() {
                return Err(D::Interrupted);
            }
            if previous.is_some_and(|before| before >= step.node) {
                return Err(D::UnorderedRegion);
            }
            previous = Some(step.node);
            recorded = recorded.checked_add(1).ok_or(D::CountOverflow)?;
            if frontier.root.is_none() {
                return Err(D::MissingRoot);
            }
            if (frontier.disposition)(step.node) != Some(step.disposition) {
                return Err(D::MissingRegion);
            }
            let region = (frontier.region)(step.node).ok_or(D::MissingRegion)?;
            if Some(step.node) == frontier.root {
                root_seen = true;
                if region.parent.is_some() {
                    return Err(D::InvalidParent);
                }
            } else {
                let parent = region.parent.ok_or(D::InvalidParent)?;
                if !(frontier.disposition)(parent).is_some_and(|d| d.expanded()) {
                    return Err(D::InvalidParent);
                }
                if !(frontier.region)(parent)
                    .ok_or(D::MissingRegion)?
                    .children
                    .contains(&Some(step.node))
                {
                    return Err(D::InvalidParent);
                }
            }
            if step.disposition == ManagedCoverageDisposition::Pruned {
                if !frontier.allow_pruning {
                    return Err(D::PruningUnavailable);
                }
            } else {
                if region.has_source
                    != matches!(
                        step.disposition,
                        ManagedCoverageDisposition::Matched | ManagedCoverageDisposition::Excluded
                    )
                {
                    return Err(D::SourceDisposition);
                }
                for child in region.children.into_iter().flatten() {
                    if (frontier.disposition)(child).is_none() {
                        return Err(D::MissingChild);
                    }
                }
            }
            if step.disposition == ManagedCoverageDisposition::Matched {
                matched = matched.checked_add(1).ok_or(D::CountOverflow)?;
            }
        }
        if !root_seen {
            return Err(D::MissingRoot);
        }
        if recorded != frontier.recorded_regions {
            return Err(D::IncompleteFrontier);
        }
        if !(frontier.checkpoint)() {
            return Err(D::Interrupted);
        }
        Ok(AdmittedManagedIndexCoverage {
            owner: Arc::clone(&self.0),
            matched,
        })
    }
}
impl ManagedIndexCoverageAuthority {
    pub fn admits(&self, coverage: &AdmittedManagedIndexCoverage) -> bool {
        Arc::ptr_eq(&self.0, &coverage.owner)
    }
}
impl AdmittedManagedIndexCoverage {
    pub const fn matched_source_count(&self) -> usize {
        self.matched
    }
}
