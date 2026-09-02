#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiPortalStackOrdinal(u64);

/// The session-local cursor for Portal activation order.
///
/// The cursor is deliberately move-only. A Portal installation may be
/// replaced, but the active session must retain this issuer so a later
/// installation cannot reuse an ordinal minted earlier in the session.
#[derive(Debug)]
pub(crate) struct UiPortalStackOrdinalIssuer {
    next: Option<UiPortalStackOrdinal>,
}

impl UiPortalStackOrdinal {
    pub(super) const fn minted(value: u64) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u64 {
        self.0
    }
}

impl UiPortalStackOrdinalIssuer {
    pub(crate) const fn new() -> Self {
        Self {
            next: Some(UiPortalStackOrdinal::minted(1)),
        }
    }

    pub(super) const fn exhausted() -> Self {
        Self { next: None }
    }

    pub(super) const fn next(&self) -> Option<UiPortalStackOrdinal> {
        self.next
    }

    /// Consumes exactly the ordinal that preparation observed. This check is
    /// performed before any Portal record is changed, so a forged or stale
    /// prepared ordinal cannot advance the session cursor.
    pub(super) fn advance(
        &mut self,
        issued: UiPortalStackOrdinal,
    ) -> Result<(), UiPortalStackOrdinal> {
        if self.next != Some(issued) {
            return Err(issued);
        }
        self.next = issued
            .value()
            .checked_add(1)
            .map(UiPortalStackOrdinal::minted);
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn force_next(&mut self, next: u64) {
        self.next = Some(UiPortalStackOrdinal::minted(next));
    }

    #[cfg(test)]
    pub(super) const fn is_exhausted(&self) -> bool {
        self.next.is_none()
    }
}
