//! Immutable relationships of one recorded correction; descriptive coordinates grant no authority.
use super::{WorthQueryAftermathCausalRole, WorthQueryRecordedCorrectionSource};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Immutable lineage recorded by one performed reversal or reapplication.
///
/// The original lineage root, the chosen historical alternative and the active
/// reversal consumed by reapplication have distinct meanings. An older chosen
/// alternative may differ from the active reversal. These relationships describe
/// committed meaning; publishing another correction requires fresh admission.
pub struct WorthQueryCorrectionLineageBinding {
    root: WorthQueryRecordedCorrectionSource,
    transition: Transition,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Transition {
    Reverse {
        prior: Option<WorthQueryRecordedCorrectionSource>,
    },
    Reapply {
        selected: WorthQueryRecordedCorrectionSource,
        active: WorthQueryRecordedCorrectionSource,
    },
}
impl WorthQueryCorrectionLineageBinding {
    pub(crate) fn reverse(
        root: WorthQueryRecordedCorrectionSource,
        prior: Option<WorthQueryRecordedCorrectionSource>,
    ) -> Self {
        Self {
            root,
            transition: Transition::Reverse { prior },
        }
    }
    pub(crate) fn reapply(
        root: WorthQueryRecordedCorrectionSource,
        selected: WorthQueryRecordedCorrectionSource,
        active: WorthQueryRecordedCorrectionSource,
    ) -> Self {
        Self {
            root,
            transition: Transition::Reapply { selected, active },
        }
    }
    /// The original ordinary occurrence inherited by every transition in this lineage.
    pub fn root(&self) -> &WorthQueryRecordedCorrectionSource {
        &self.root
    }
    /// The historical reversal chosen for a reapplication, absent for a reversal.
    pub fn selected_alternative(&self) -> Option<&WorthQueryRecordedCorrectionSource> {
        match &self.transition {
            Transition::Reverse { .. } => None,
            Transition::Reapply { selected, .. } => Some(selected),
        }
    }
    /// The exact reversal consumed by this reapplication, independent of the chosen alternative.
    pub fn active_reversal(&self) -> Option<&WorthQueryRecordedCorrectionSource> {
        match &self.transition {
            Transition::Reverse { .. } => None,
            Transition::Reapply { active, .. } => Some(active),
        }
    }
    /// The performed reapplication being reversed, or the reversal being consumed.
    /// The first reversal of an ordinary occurrence has no prior transition.
    pub fn prior_transition(&self) -> Option<&WorthQueryRecordedCorrectionSource> {
        match &self.transition {
            Transition::Reverse { prior } => prior.as_ref(),
            Transition::Reapply { active, .. } => Some(active),
        }
    }
    /// Whether this immutable relationship records reversal or reapplication.
    pub fn role(&self) -> WorthQueryAftermathCausalRole {
        match self.transition {
            Transition::Reverse { .. } => WorthQueryAftermathCausalRole::RecordedReverse,
            Transition::Reapply { .. } => WorthQueryAftermathCausalRole::RecordedReapply,
        }
    }
}
