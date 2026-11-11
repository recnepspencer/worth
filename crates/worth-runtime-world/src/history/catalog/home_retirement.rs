//! Retire one exact original history row without destroying its allocation.
use super::*;
use crate::lifecycle::RuntimeWorldCloseDenial as Denial;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RelationalBranchBasisRetirementDenial as NativeDenial, runtime::RelationalRuntime,
};
#[derive(Debug)]
pub(crate) struct HistoryHomeRetirement {
    identity: CompositeCommitIdentity,
    components: crate::retention::component_obligation::RetiringHistoryComponents,
    commit: crate::history::CompositeCommitRetirement,
    publication: Option<PublicationHomeRetirement>,
}
#[derive(Debug)]
enum PublicationHomeRetirement {
    Live(crate::history::PublicationFactsRetirement),
    Readmitted(crate::history::ReadmittedPublicationRetirement),
}
impl PublicationHomeRetirement {
    fn new(original: crate::history::PublicationEvidence) -> Self {
        match original {
            crate::history::PublicationEvidence::Live(original) => {
                Self::Live(crate::history::PublicationFactsRetirement::new(original))
            }
            crate::history::PublicationEvidence::Readmitted(original) => Self::Readmitted(
                crate::history::ReadmittedPublicationRetirement::new(original),
            ),
        }
    }
    fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, Denial> {
        match self {
            Self::Live(original) => {
                original
                    .advance(request, native)
                    .map_err(|cause| match cause {
                        NativeDenial::Busy => Denial::Busy,
                        NativeDenial::Execution(cause) => Denial::Execution(cause),
                        NativeDenial::ForeignOwner => Denial::CloseProgressInvariant,
                    })
            }
            Self::Readmitted(original) => original.advance(request, native),
        }
    }
}
impl HistoryHomeRetirement {
    pub(crate) fn identity(&self) -> &CompositeCommitIdentity {
        &self.identity
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, Denial> {
        if !self.components.advance(request)? {
            return Ok(false);
        }
        if !self
            .commit
            .advance(request, native)
            .map_err(|cause| match cause {
                NativeDenial::Busy => Denial::Busy,
                NativeDenial::Execution(cause) => Denial::Execution(cause),
                NativeDenial::ForeignOwner => Denial::CloseProgressInvariant,
            })?
        {
            return Ok(false);
        }
        match &mut self.publication {
            Some(original) => original.advance(request, native),
            None => Ok(true),
        }
    }
}
impl CompositeHistoryCatalog {
    pub(crate) fn try_begin_next_home_retirement(
        &self,
        after: Option<&CompositeCommitIdentity>,
    ) -> Result<Option<HistoryHomeRetirement>, Denial> {
        let mut state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        let identity = state
            .inspection_order
            .range((
                after.map_or(std::ops::Bound::Unbounded, std::ops::Bound::Excluded),
                std::ops::Bound::Unbounded,
            ))
            .next()
            .cloned();
        let Some(identity) = identity else {
            return Ok(None);
        };
        let slot = state
            .entries
            .get_mut(&identity)
            .ok_or(Denial::CloseProgressInvariant)?;
        let entry = Arc::get_mut(slot)
            .and_then(OnceLock::get_mut)
            .ok_or(Denial::Busy)?;
        if entry.commit.is_none() {
            return Err(Denial::CloseProgressInvariant);
        }
        let components = entry
            .pins
            .try_begin_close_retirement()
            .map_err(|()| Denial::Busy)?;
        // No owner operation runs under the catalog guard. Only original shares
        // move into the one prepaid pending phase in the real close task.
        let commit = entry
            .commit
            .take()
            .expect("original serving history commit");
        Ok(Some(HistoryHomeRetirement {
            identity,
            components,
            commit: crate::history::CompositeCommitRetirement::new(commit),
            publication: entry.publication.take().map(PublicationHomeRetirement::new),
        }))
    }
    pub(crate) fn try_home_retirement_count(&self) -> Result<usize, Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        Ok(state.inspection_order.len())
    }
}
