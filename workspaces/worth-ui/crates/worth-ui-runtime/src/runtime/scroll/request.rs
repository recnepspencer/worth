pub(in crate::runtime) const UI_SCROLL_CHAIN_DEPTH_LIMIT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChainEntry {
    owner: super::UiScrollOwnerIdentity,
    incarnation: super::UiScrollOwnerIncarnation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollDeltaCause {
    Host {
        source: worth_ui_host_contract::UiHostScrollDeltaSource,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
    },
    ProgrammaticReveal,
    /// The accepted sample of a settling Scroll content group reached the
    /// displayed pose, and the semantic offset is being brought to where the
    /// host has already put the content. No reader gesture is asking for this:
    /// it is the settle's own truth arriving one accepted frame at a time.
    AcceptedSampleSettlement,
    /// A reader is dragging the scroll thumb. The offset is placed directly,
    /// with no smoothing and no transition: the thumb must stay under the
    /// pointer, so the delta the drag names is the delta that applies.
    ChromeThumbDrag,
    /// A reader pressed the chrome track beside the thumb, and the region paged
    /// toward the press by one viewport less one line.
    ChromeTrackPage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollDeltaRequest {
    chain: Vec<UiScrollChainEntry>,
    delta: super::UiScrollDelta,
    cause: UiScrollDeltaCause,
}

impl UiScrollChainEntry {
    pub(crate) const fn new(
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
    ) -> Self {
        Self { owner, incarnation }
    }

    pub(crate) const fn owner(self) -> super::UiScrollOwnerIdentity {
        self.owner
    }

    pub(crate) const fn incarnation(self) -> super::UiScrollOwnerIncarnation {
        self.incarnation
    }
}

impl UiScrollDeltaRequest {
    pub(crate) fn new(
        chain: Vec<UiScrollChainEntry>,
        delta: super::UiScrollDelta,
        cause: UiScrollDeltaCause,
    ) -> Result<Self, super::UiScrollRouteDenial> {
        if chain.is_empty() {
            return Err(super::UiScrollRouteDenial::EmptyChain);
        }
        if chain.len() > UI_SCROLL_CHAIN_DEPTH_LIMIT {
            return Err(super::UiScrollRouteDenial::ChainDepthExceeded);
        }
        for (index, entry) in chain.iter().enumerate() {
            if chain[..index]
                .iter()
                .any(|predecessor| predecessor.owner == entry.owner)
            {
                return Err(super::UiScrollRouteDenial::OwnershipCycle);
            }
        }
        Ok(Self {
            chain,
            delta,
            cause,
        })
    }

    pub(super) fn chain(&self) -> &[UiScrollChainEntry] {
        &self.chain
    }

    pub(in crate::runtime) const fn delta(&self) -> super::UiScrollDelta {
        self.delta
    }

    pub(in crate::runtime) const fn cause(&self) -> UiScrollDeltaCause {
        self.cause
    }
}
