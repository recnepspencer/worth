//! The two spaces a frame's geometry is in: where layout put it, and where
//! the frame presents it.
//!
//! The same box type serves both, so nothing but these wrappers tells them
//! apart. Geometry leaves layout as [`UiLaidOut`]; only a placement makes
//! it [`UiPresented`], so what the host paints, hits and reads is geometry a
//! placement has presented, and laid-out geometry is read in layout space
//! only by name. The methods that yield presented geometry outside a
//! placement are mints whose callers road1.toml declares.

/// Geometry where layout put it, before any Portal presents it elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiLaidOut<T>(T);

impl<T> UiLaidOut<T> {
    /// Geometry a layout pass produced.
    pub(crate) const fn from_layout(value: T) -> Self {
        Self(value)
    }

    /// This geometry read where layout put it: to relate it to other laid-out
    /// geometry, never to show it.
    pub(crate) const fn in_layout_space(&self) -> &T {
        &self.0
    }

    /// This geometry taken where layout put it: to relate it to other
    /// laid-out geometry, never to show it.
    pub(crate) fn into_layout_space(self) -> T {
        self.0
    }

    /// This geometry shown where layout put it, as the placement
    /// [`super::UiMountedPlacement::InPlace`] presents it: for an occurrence
    /// no Portal presents.
    pub(crate) fn in_place(self) -> UiPresented<T> {
        UiPresented(self.0)
    }

    /// This geometry changed within layout space.
    pub(crate) fn map<U>(self, change: impl FnOnce(T) -> U) -> UiLaidOut<U> {
        UiLaidOut(change(self.0))
    }

    /// This geometry changed within layout space, where the change may fail.
    pub(crate) fn try_map<U, E>(
        self,
        change: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<UiLaidOut<U>, E> {
        change(self.0).map(UiLaidOut)
    }
}

/// Geometry where a frame presents it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiPresented<T>(T);

impl<T> UiPresented<T> {
    /// Only a placement presents geometry.
    pub(super) const fn placed(value: T) -> Self {
        Self(value)
    }

    /// This geometry where the frame shows it.
    pub(crate) const fn shown(&self) -> &T {
        &self.0
    }

    /// This geometry taken where the frame shows it.
    pub(crate) fn into_shown(self) -> T {
        self.0
    }

    /// This geometry changed where the frame shows it.
    pub(crate) fn map<U>(self, change: impl FnOnce(T) -> U) -> UiPresented<U> {
        UiPresented(change(self.0))
    }
}
