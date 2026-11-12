//! A failed durable transition retains its actual linear handle and owner cause.
use super::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
#[derive(Debug)]
pub enum WorthQueryRecoveryTransitionDenial {
    Retained {
        denial: WorthQueryRecoveryHandleDenial,
        handle: Box<WorthQueryRecoveryHandle>,
    },
    Relinquished(WorthQueryRecoveryHandleDenial),
    Terminal(WorthQueryRecoveryHandleDenial),
}
impl WorthQueryRecoveryTransitionDenial {
    pub fn denial(&self) -> &WorthQueryRecoveryHandleDenial {
        match self {
            Self::Retained { denial, .. } | Self::Relinquished(denial) | Self::Terminal(denial) => {
                denial
            }
        }
    }
    pub fn kind(&self) -> WorthQueryRecoveryHandleDenialKind {
        match self {
            Self::Retained { denial, .. } | Self::Relinquished(denial) | Self::Terminal(denial) => {
                denial.kind()
            }
        }
    }
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryRecoveryHandleDenial,
        Option<WorthQueryRecoveryHandle>,
    ) {
        match self {
            Self::Retained { denial, handle } => (denial, Some(*handle)),
            Self::Relinquished(denial) | Self::Terminal(denial) => (denial, None),
        }
    }
}
impl From<WorthQueryRecoveryHandleDenial> for WorthQueryRecoveryTransitionDenial {
    fn from(denial: WorthQueryRecoveryHandleDenial) -> Self {
        Self::Relinquished(denial)
    }
}
