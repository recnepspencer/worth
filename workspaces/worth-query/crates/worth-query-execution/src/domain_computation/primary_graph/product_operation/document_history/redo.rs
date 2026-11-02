//! Admitted redo hints describe completed alternatives; they grant no navigation authority.
use super::{WorthQueryHistoryNodeId, WorthQueryHistoryPage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryRedoChoice {
    NoChild,
    Single(WorthQueryHistoryNodeId),
    ChoiceRequired,
}

/// A fixed bounded classification plus a separately bounded child-page walk.
/// Even a single child must be explicitly supplied to fresh `redo_to` admission.
#[derive(Debug)]
pub struct WorthQueryHistoryRedoChoices {
    pub(super) choice: WorthQueryHistoryRedoChoice,
    pub(super) children: WorthQueryHistoryPage,
}

impl WorthQueryHistoryRedoChoices {
    pub fn choice(&self) -> WorthQueryHistoryRedoChoice {
        self.choice
    }

    /// Limits and examined entries describe this page, excluding the fixed
    /// first-two classification and last-child boundary probes.
    pub fn children(&self) -> &WorthQueryHistoryPage {
        &self.children
    }
}
