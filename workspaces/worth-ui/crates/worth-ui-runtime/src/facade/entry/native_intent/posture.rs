#[must_use]
pub struct WorthUiNativeIntentPosture {
    pub(in crate::facade::entry) observation: crate::mounting::UiIntentPostureObservation,
    pub(in crate::facade::entry) commit: crate::mounting::UiIntentPostureCommit,
    pub(super) kind: WorthUiNativeIntentPostureKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiNativeIntentPostureKind {
    Admitted,
    ConfirmationRequired,
    Completed,
    Denied,
    StaleConfirmation,
    Cancelled,
}

impl WorthUiNativeIntentPosture {
    pub(in crate::facade::entry) fn new(
        observation: crate::mounting::UiIntentPostureObservation,
        commit: crate::mounting::UiIntentPostureCommit,
        kind: crate::fact_contract::UiIntentPostureKind,
    ) -> Self {
        Self {
            observation,
            commit,
            kind: kind.into(),
        }
    }

    pub const fn kind(&self) -> WorthUiNativeIntentPostureKind {
        self.kind
    }

    /// The observed target identifies product feedback; it grants no interaction authority.
    pub const fn mounted_instance(&self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.observation.mounted_instance()
    }
}

impl From<crate::fact_contract::UiIntentPostureKind> for WorthUiNativeIntentPostureKind {
    fn from(kind: crate::fact_contract::UiIntentPostureKind) -> Self {
        match kind {
            crate::fact_contract::UiIntentPostureKind::Admitted => Self::Admitted,
            crate::fact_contract::UiIntentPostureKind::ConfirmationRequired => {
                Self::ConfirmationRequired
            }
            crate::fact_contract::UiIntentPostureKind::Completed => Self::Completed,
            crate::fact_contract::UiIntentPostureKind::Denied => Self::Denied,
            crate::fact_contract::UiIntentPostureKind::StaleConfirmation => Self::StaleConfirmation,
            crate::fact_contract::UiIntentPostureKind::Cancelled => Self::Cancelled,
        }
    }
}
