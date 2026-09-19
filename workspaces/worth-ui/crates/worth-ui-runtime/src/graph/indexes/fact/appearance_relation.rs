use crate::declaration::UiAspectName;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiGraphFactConsumptionRelation {
    General {
        affected_aspect: Option<UiAspectName>,
    },
    AppearanceRoleSlot {
        role: worth_ui_dsl::UiAppearanceRoleIdentity,
        revision: worth_ui_dsl::UiAppearanceRoleRevision,
        aspect: worth_ui_dsl::UiAppearanceAspect,
        requested_slot: Box<str>,
        terminal_slot: Option<Box<str>>,
    },
}

impl UiGraphFactConsumptionRelation {
    pub(crate) const fn general(affected_aspect: Option<UiAspectName>) -> Self {
        Self::General { affected_aspect }
    }

    pub(crate) fn appearance_role_slot(
        role: worth_ui_dsl::UiAppearanceRoleIdentity,
        revision: worth_ui_dsl::UiAppearanceRoleRevision,
        aspect: worth_ui_dsl::UiAppearanceAspect,
        requested_slot: impl Into<Box<str>>,
        terminal_slot: Option<Box<str>>,
    ) -> Self {
        Self::AppearanceRoleSlot {
            role,
            revision,
            aspect,
            requested_slot: requested_slot.into(),
            terminal_slot,
        }
    }

    pub(crate) const fn affected_aspect(&self) -> Option<&UiAspectName> {
        match self {
            Self::General { affected_aspect } => affected_aspect.as_ref(),
            Self::AppearanceRoleSlot { .. } => None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn is_appearance_role_slot(&self) -> bool {
        matches!(self, Self::AppearanceRoleSlot { .. })
    }

    pub(crate) fn matches_theme_token(
        &self,
        capability_identity: &str,
        authored_identity: &str,
    ) -> bool {
        let matches =
            |candidate: &str| candidate == capability_identity || candidate == authored_identity;
        match self {
            Self::General { .. } => false,
            Self::AppearanceRoleSlot {
                requested_slot,
                terminal_slot,
                ..
            } => matches(requested_slot) || terminal_slot.as_deref().is_some_and(matches),
        }
    }
}
