use worth_ui_dsl::UiAppearanceRoleIdentity;

/// Which axes a scroll region shows chrome for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiScrollAxisSupport {
    /// The inline (horizontal) axis only.
    Inline,
    /// The block (vertical) axis only.
    Block,
    /// Both axes.
    Both,
}

/// Why a scroll chrome contract was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiScrollChromeContractDenial {
    /// The track and the thumb named the same appearance role. Two parts that
    /// paint differently cannot share one role identity.
    RoleIdentitiesCollide,
    /// A named appearance role is not registered. Raised where the descriptor
    /// meets the appearance role registry, not at construction: this contract
    /// carries identities and holds no registry.
    UnregisteredRole,
    /// The region declares chrome for an axis it does not own. Raised where the
    /// descriptor meets its scroll ownership, not at construction.
    AxisNotOwned,
}

/// A scroll region's declared chrome: which axes it shows, and which registered
/// appearance roles paint the track and the thumb.
///
/// This is a declaration and nothing else. It carries no geometry, no offset
/// and no live authority; whoever derives chrome rectangles reads it and owns
/// the result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiScrollChromeContract {
    axes: UiScrollAxisSupport,
    track_role: UiAppearanceRoleIdentity,
    thumb_role: UiAppearanceRoleIdentity,
}

impl UiScrollChromeContract {
    pub fn new(
        axes: UiScrollAxisSupport,
        track_role: UiAppearanceRoleIdentity,
        thumb_role: UiAppearanceRoleIdentity,
    ) -> Result<Self, UiScrollChromeContractDenial> {
        if track_role == thumb_role {
            return Err(UiScrollChromeContractDenial::RoleIdentitiesCollide);
        }
        Ok(Self {
            axes,
            track_role,
            thumb_role,
        })
    }

    pub const fn axes(&self) -> UiScrollAxisSupport {
        self.axes
    }

    pub fn track_role(&self) -> UiAppearanceRoleIdentity {
        self.track_role.clone()
    }

    pub fn thumb_role(&self) -> UiAppearanceRoleIdentity {
        self.thumb_role.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{UiScrollAxisSupport, UiScrollChromeContract, UiScrollChromeContractDenial};
    use worth_ui_dsl::UiAppearanceRoleIdentity;

    fn role(text: &str) -> UiAppearanceRoleIdentity {
        UiAppearanceRoleIdentity::new(text).expect("the fixture role identity is well formed")
    }

    /// A track and a thumb are two painted parts, so they are two roles.
    #[test]
    fn one_role_cannot_paint_both_the_track_and_the_thumb() {
        assert_eq!(
            UiScrollChromeContract::new(
                UiScrollAxisSupport::Both,
                role("fixture.scroll_track"),
                role("fixture.scroll_track"),
            ),
            Err(UiScrollChromeContractDenial::RoleIdentitiesCollide)
        );
    }

    /// The contract hands back exactly the axes and roles it was given.
    #[test]
    fn an_admitted_contract_preserves_its_declared_axes_and_roles() {
        let contract = UiScrollChromeContract::new(
            UiScrollAxisSupport::Block,
            role("fixture.scroll_track"),
            role("fixture.scroll_thumb"),
        )
        .expect("distinct roles are admitted");

        assert_eq!(contract.axes(), UiScrollAxisSupport::Block);
        assert_eq!(contract.track_role(), role("fixture.scroll_track"));
        assert_eq!(contract.thumb_role(), role("fixture.scroll_thumb"));
    }
}
