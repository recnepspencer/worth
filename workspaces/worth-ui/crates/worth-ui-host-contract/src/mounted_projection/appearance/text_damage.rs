//! Unresolved span coverage required by a mounted text paint change.

use super::{
    UiMountedAppearanceMechanicChange, UiMountedAppearanceMechanicIdentity, UiMountedAppearanceWork,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceTextDamageTransition {
    Insert,
    Replace,
    Remove,
}

/// A semantic damage requirement, not a completed physical rectangle.
/// Presentation must join the exact target/span to candidate glyph images and
/// accepted predecessor coverage before admitting physical text damage.
/// The requirement must remain associated with its enclosing work/fragment's
/// receipt, binding, and text-candidate affinity; it grants no lookup authority
/// against arbitrary current text and is not a finalized presentation proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceTextDamageRequirement {
    target: crate::UiMountedInstanceIdentity,
    command: crate::UiMountedPaintCommandIdentity,
    span: crate::UiMountedTextPaintSpanIdentity,
    transition: UiAppearanceTextDamageTransition,
}

impl UiAppearanceTextDamageRequirement {
    pub const fn target(self) -> crate::UiMountedInstanceIdentity {
        self.target
    }
    pub const fn span(self) -> crate::UiMountedTextPaintSpanIdentity {
        self.span
    }
    pub const fn command(self) -> crate::UiMountedPaintCommandIdentity {
        self.command
    }
    pub const fn transition(self) -> UiAppearanceTextDamageTransition {
        self.transition
    }

    fn from_change(change: &UiMountedAppearanceMechanicChange) -> Option<Self> {
        let (identity, transition) = match change {
            UiMountedAppearanceMechanicChange::Insert(mechanic) => (
                mechanic.identity(),
                UiAppearanceTextDamageTransition::Insert,
            ),
            UiMountedAppearanceMechanicChange::Replace { predecessor, .. } => (
                predecessor.clone(),
                UiAppearanceTextDamageTransition::Replace,
            ),
            UiMountedAppearanceMechanicChange::Remove(predecessor) => (
                predecessor.clone(),
                UiAppearanceTextDamageTransition::Remove,
            ),
        };
        let UiMountedAppearanceMechanicIdentity::TextForeground {
            target,
            command,
            span,
        } = identity
        else {
            return None;
        };
        Some(Self {
            target,
            command,
            span,
            transition,
        })
    }
}

impl UiMountedAppearanceWork {
    /// Derives unresolved text coverage requirements from the canonical delta.
    /// `damage()` contains completed non-text logical regions; these requirements
    /// must not be treated as empty damage or replaced with node allocations.
    pub fn text_damage_requirements(
        &self,
    ) -> impl Iterator<Item = UiAppearanceTextDamageRequirement> + '_ {
        self.changes()
            .iter()
            .filter_map(UiAppearanceTextDamageRequirement::from_change)
    }
}
