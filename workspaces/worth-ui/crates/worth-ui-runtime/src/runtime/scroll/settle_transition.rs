//! `UiPreparedScrollSettleTransition`: one Scroll owner's staged settle, proven
//! far enough to be published.
//!
//! Staging a settle target and lowering it to a Motion request are two separate
//! acts, and neither of them is authority to publish. This type is what the two
//! together produce: an exact owner occurrence, the target it is settling
//! toward, the Motion request that interpolates it, and the route revision that
//! names this settle apart from every other one. Nothing here samples, and
//! nothing here writes an offset.

use core::num::NonZeroU64;

/// Why a staged settle could not become a publishable transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollSettleTransitionDenial {
    /// The Scroll route revision this settle would be identified by is zero, so
    /// it names no request. A routed receipt always carries a later revision, so
    /// this is reachable only from an unrouted one.
    RequestLineageUnavailable,
}

#[must_use = "a prepared Scroll settle transition must be published or dropped deliberately"]
pub(crate) struct UiPreparedScrollSettleTransition {
    target: super::transition::UiScrollTransitionTarget,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    request: crate::runtime::motion::UiMotionTransitionRequest,
    lineage: NonZeroU64,
}

impl UiPreparedScrollSettleTransition {
    /// `lineage` is the revision of the Scroll route this observation produced.
    /// It is monotonic per session, so it separates successive settles of one
    /// owner without a second identity issuer.
    pub(crate) fn prepare(
        target: super::transition::UiScrollTransitionTarget,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        request: crate::runtime::motion::UiMotionTransitionRequest,
        lineage: u64,
    ) -> Result<Self, UiScrollSettleTransitionDenial> {
        let lineage = NonZeroU64::new(lineage)
            .ok_or(UiScrollSettleTransitionDenial::RequestLineageUnavailable)?;
        Ok(Self {
            target,
            mounted_instance,
            request,
            lineage,
        })
    }

    pub(in crate::runtime) const fn request_lineage(&self) -> NonZeroU64 {
        self.lineage
    }

    pub(in crate::runtime) fn semantic_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.target.owner().semantic_surface()
    }

    pub(in crate::runtime) const fn motion_request(
        &self,
    ) -> crate::runtime::motion::UiMotionTransitionRequest {
        self.request
    }

    /// The occupancy scope this settle occupies: the mounted occurrence whose
    /// scrolled content group is moving. Motion derives the same scope from the
    /// request's target, so Scroll and Motion occupy one owner between them.
    pub(in crate::runtime) const fn scope(
        &self,
    ) -> crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity {
        crate::runtime::session::service_proposal::
            UiServiceProposalOccupancyScopeIdentity::for_mounted_owner(self.mounted_instance)
    }
}
