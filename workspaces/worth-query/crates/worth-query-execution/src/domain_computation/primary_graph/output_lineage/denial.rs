use super::super::WorthQueryApplicationOutputProjectionDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPriorOutputDenialKind {
    Unavailable,
    UndeclaredFamily,
    MissingRole,
    ActionMismatch,
    EntityMismatch,
    OutputUnavailable,
    UndeclaredDecisionTarget,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPriorOutputDenial {
    kind: WorthQueryPriorOutputDenialKind,
    role: String,
}

impl WorthQueryPriorOutputDenial {
    pub const fn kind(&self) -> WorthQueryPriorOutputDenialKind {
        self.kind
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryPriorOutputDenialKind,
        role: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            role: role.into(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn projection(
        role: &str,
        denial: WorthQueryApplicationOutputProjectionDenial,
    ) -> Self {
        let kind = match denial {
            WorthQueryApplicationOutputProjectionDenial::MissingRole => {
                WorthQueryPriorOutputDenialKind::MissingRole
            }
            WorthQueryApplicationOutputProjectionDenial::ForeignBinding => {
                WorthQueryPriorOutputDenialKind::Unavailable
            }
            WorthQueryApplicationOutputProjectionDenial::ActionMismatch => {
                WorthQueryPriorOutputDenialKind::ActionMismatch
            }
            WorthQueryApplicationOutputProjectionDenial::EntityMismatch => {
                WorthQueryPriorOutputDenialKind::EntityMismatch
            }
        };
        Self::new(kind, role)
    }
}

impl std::fmt::Display for WorthQueryPriorOutputDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "prior output denied: {:?} ({})",
            self.kind, self.role
        )
    }
}

impl std::error::Error for WorthQueryPriorOutputDenial {}
