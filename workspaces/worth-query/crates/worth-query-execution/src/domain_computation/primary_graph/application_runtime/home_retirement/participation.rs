//! Preserve original participation proof while retiring its provider invocations.
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    provider_anchor::WorthQueryGraphProviderAnchor,
    provider_home_retirement::RetiredPrimaryProviderAnchor,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::{
    RetiredQueryGraphParticipation, WorthQueryInstalledGraphParticipationAuthority,
};

pub(in crate::domain_computation::primary_graph) struct PrimaryGraphParticipationCustody {
    original: Option<WorthQueryInstalledGraphParticipationAuthority>,
    anchor_share: Option<Arc<WorthQueryGraphProviderAnchor>>,
    anchor: Option<WorthQueryGraphProviderAnchor>,
    _retired_proof: Option<RetiredQueryGraphParticipation>,
    retired_anchor: Option<RetiredPrimaryProviderAnchor>,
}
impl PrimaryGraphParticipationCustody {
    pub(in crate::domain_computation::primary_graph) fn new(
        original: WorthQueryInstalledGraphParticipationAuthority,
    ) -> Self {
        Self {
            original: Some(original),
            anchor_share: None,
            anchor: None,
            _retired_proof: None,
            retired_anchor: None,
        }
    }
    pub(super) fn preflight(
        &mut self,
        expected: &Arc<WorthQueryPrimaryGraphProvider>,
    ) -> Result<(), Denial> {
        let original = self
            .original
            .as_mut()
            .expect("only serving participation preflights");
        original
            .preflight_provider_retirement()
            .map_err(|_| Denial::Busy(Owner::GraphProvider))?;
        let anchor = original
            .retain_provider_anchor::<WorthQueryGraphProviderAnchor>()
            .ok_or(Denial::Unsupported(Owner::GraphProvider))?;
        anchor.preflight_home_retirement(expected)
    }
    pub(super) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        expected: &Arc<WorthQueryPrimaryGraphProvider>,
    ) -> Result<Progress, Denial> {
        if self.retired_anchor.is_some() {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if let Some(original) = self.original.take() {
            match original.into_provider_retirement::<WorthQueryGraphProviderAnchor>() {
                Ok((anchor, retired)) => {
                    self.anchor_share = Some(anchor);
                    self._retired_proof = Some(retired);
                }
                Err(original) => {
                    self.original = Some(original);
                    return Err(Denial::Unsupported(Owner::GraphProvider));
                }
            }
            return Ok(Progress::Advanced);
        }
        if let Some(anchor) = self.anchor_share.take() {
            match Arc::try_unwrap(anchor) {
                Ok(anchor) => self.anchor = Some(anchor),
                Err(anchor) => {
                    self.anchor_share = Some(anchor);
                    return Err(Denial::Busy(Owner::GraphProvider));
                }
            }
            return Ok(Progress::Advanced);
        }
        match self
            .anchor
            .take()
            .expect("same original provider anchor")
            .retire_home_aliases(expected, request)
        {
            Ok(retired) => self.retired_anchor = Some(retired),
            Err((anchor, cause)) => {
                self.anchor = Some(anchor);
                return Err(cause);
            }
        }
        Ok(Progress::Complete)
    }
}
impl std::ops::Deref for PrimaryGraphParticipationCustody {
    type Target = WorthQueryInstalledGraphParticipationAuthority;
    fn deref(&self) -> &Self::Target {
        self.original
            .as_ref()
            .expect("retired Query exposes no participation")
    }
}
