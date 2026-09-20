#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollRouteDenial {
    EmptyChain,
    ChainDepthExceeded,
    OwnershipCycle,
    UnknownOwner,
    StaleOwnerIncarnation,
    CrossSurfaceChain,
    InitialOffsetOutOfBounds,
    RevisionExhausted,
    CounterOverflow,
    /// The distance between the offset an owner holds and the one a route asks
    /// it to hold is not representable, so no delta names the move.
    OffsetDeltaOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChainTransition {
    owner: super::UiScrollOwnerIdentity,
    previous: super::UiScrollOffset,
    current: super::UiScrollOffset,
    consumed: super::UiScrollDelta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollRouteReceipt {
    cause: super::UiScrollDeltaCause,
    transitions: Vec<UiScrollChainTransition>,
    remainder: super::UiScrollDelta,
    owners_visited: u16,
    revision: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiScrollCounters {
    admitted_requests: u64,
    rejected_requests: u64,
    owners_visited: u64,
    owners_changed: u64,
}

impl UiScrollChainTransition {
    pub(super) const fn new(
        owner: super::UiScrollOwnerIdentity,
        previous: super::UiScrollOffset,
        current: super::UiScrollOffset,
        consumed: super::UiScrollDelta,
    ) -> Self {
        Self {
            owner,
            previous,
            current,
            consumed,
        }
    }

    pub(crate) const fn owner(self) -> super::UiScrollOwnerIdentity {
        self.owner
    }

    pub(in crate::runtime) const fn previous(self) -> super::UiScrollOffset {
        self.previous
    }

    pub(crate) const fn current(self) -> super::UiScrollOffset {
        self.current
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn consumed(self) -> super::UiScrollDelta {
        self.consumed
    }
}

impl UiScrollRouteReceipt {
    pub(super) fn new(
        cause: super::UiScrollDeltaCause,
        transitions: Vec<UiScrollChainTransition>,
        remainder: super::UiScrollDelta,
        owners_visited: u16,
        revision: u64,
    ) -> Self {
        Self {
            cause,
            transitions,
            remainder,
            owners_visited,
            revision,
        }
    }

    /// What asked for this route: a host delta, a chrome track page, or an
    /// accepted-sample write-back. Certification reads it to tell them apart.
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn cause(&self) -> super::UiScrollDeltaCause {
        self.cause
    }

    pub(crate) fn transitions(&self) -> &[UiScrollChainTransition] {
        &self.transitions
    }

    pub(crate) const fn remainder(&self) -> super::UiScrollDelta {
        self.remainder
    }

    pub(crate) const fn owners_visited(&self) -> u16 {
        self.owners_visited
    }

    /// The session-monotonic revision this route produced. A settle is named
    /// by it, so every notch carries its own service request identity.
    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }
}

impl UiScrollCounters {
    pub(super) const fn new() -> Self {
        Self {
            admitted_requests: 0,
            rejected_requests: 0,
            owners_visited: 0,
            owners_changed: 0,
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) const fn admitted_requests(self) -> u64 {
        self.admitted_requests
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) const fn rejected_requests(self) -> u64 {
        self.rejected_requests
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn owners_visited(self) -> u64 {
        self.owners_visited
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn owners_changed(self) -> u64 {
        self.owners_changed
    }

    pub(super) fn after_admission(
        self,
        visited: u16,
        changed: usize,
    ) -> Result<Self, UiScrollRouteDenial> {
        Ok(Self {
            admitted_requests: self
                .admitted_requests
                .checked_add(1)
                .ok_or(UiScrollRouteDenial::CounterOverflow)?,
            rejected_requests: self.rejected_requests,
            owners_visited: self
                .owners_visited
                .checked_add(u64::from(visited))
                .ok_or(UiScrollRouteDenial::CounterOverflow)?,
            owners_changed: self
                .owners_changed
                .checked_add(changed as u64)
                .ok_or(UiScrollRouteDenial::CounterOverflow)?,
        })
    }

    pub(super) fn reject(&mut self) {
        self.rejected_requests = self.rejected_requests.saturating_add(1);
    }
}
