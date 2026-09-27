use std::rc::Rc;

use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning;
use crate::runtime::{
    WorthUiChildRangeHandle, WorthUiCommandHandle, WorthUiComponentHandle, WorthUiStateSlotHandle,
    WorthUiTokenHandle,
};
#[cfg(any(test, feature = "certification-support"))]
use crate::runtime::{WorthUiOrdinaryLanePlan, WorthUiPlanNodeInputFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiOrdinaryPlanSummaryRequest {
    Component,
    ChildRange,
    Command,
    Token,
    StateSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiOrdinarySummaryTarget {
    Component(WorthUiComponentHandle),
    ChildRange(WorthUiChildRangeHandle),
    Command(WorthUiCommandHandle),
    Token(WorthUiTokenHandle),
    StateSlot(WorthUiStateSlotHandle),
}

impl WorthUiOrdinarySummaryTarget {
    pub fn frame_target(self) -> crate::runtime::WorthUiOrdinaryFrameTarget {
        match self {
            Self::Component(handle) => {
                crate::runtime::WorthUiOrdinaryFrameTarget::component(handle)
            }
            Self::ChildRange(handle) => {
                crate::runtime::WorthUiOrdinaryFrameTarget::child_range(handle)
            }
            Self::Command(handle) => crate::runtime::WorthUiOrdinaryFrameTarget::command(handle),
            Self::Token(handle) => {
                crate::runtime::WorthUiOrdinaryFrameTarget::token_support(handle)
            }
            Self::StateSlot(handle) => {
                crate::runtime::WorthUiOrdinaryFrameTarget::state_slot(handle)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiOrdinaryPlanSummary {
    request: WorthUiOrdinaryPlanSummaryRequest,
    target: Option<WorthUiOrdinarySummaryTarget>,
    exact_meaning: Option<Rc<WorthUiPlanOrdinaryMeaning>>,
    family_row_count: usize,
    target_semantic_digest: Option<u64>,
    family_index_lookup_count: usize,
    direct_row_lookup_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiOrdinaryPlanSummaryDenial {
    ActivePlanNotOrdinaryExecutable,
    CorruptFamilyIndex,
}

impl WorthUiOrdinaryPlanSummary {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn from_plan(
        plan: &WorthUiOrdinaryLanePlan,
        request: WorthUiOrdinaryPlanSummaryRequest,
    ) -> Result<Self, WorthUiOrdinaryPlanSummaryDenial> {
        let family = request.family();
        let (family_row_count, row) = plan.first_row_for_family(family);
        if family_row_count > 0 && row.is_none() {
            return Err(WorthUiOrdinaryPlanSummaryDenial::CorruptFamilyIndex);
        }
        let target = row.as_ref().map(|row| request.target(row.runtime_handle()));
        let exact_meaning = row
            .as_ref()
            .and_then(crate::runtime::WorthUiOrdinaryLaneNode::ordinary_meaning_reference);
        let target_semantic_digest = row
            .as_ref()
            .map(crate::runtime::WorthUiOrdinaryLaneNode::ordinary_semantic_digest);
        Ok(Self {
            request,
            target,
            exact_meaning,
            family_row_count,
            target_semantic_digest,
            family_index_lookup_count: 1,
            direct_row_lookup_count: usize::from(row.is_some()),
        })
    }

    pub fn request(&self) -> WorthUiOrdinaryPlanSummaryRequest {
        self.request
    }

    pub fn target(&self) -> Option<WorthUiOrdinarySummaryTarget> {
        self.target
    }

    pub fn family_row_count(&self) -> usize {
        self.family_row_count
    }

    pub fn target_semantic_digest(&self) -> Option<u64> {
        self.target_semantic_digest
    }

    pub fn family_index_lookup_count(&self) -> usize {
        self.family_index_lookup_count
    }

    pub fn direct_row_lookup_count(&self) -> usize {
        self.direct_row_lookup_count
    }

    pub fn component_descriptor(&self) -> Option<&crate::capability::ComponentDescriptor> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::Component(value) => Some(value.descriptor()),
            WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::Token(_)
            | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
        }
    }

    pub fn child_target_count(&self) -> Option<usize> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::ChildRange(value) => Some(value.child_identities().len()),
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::Token(_)
            | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
        }
    }

    pub fn command_descriptor(&self) -> Option<&crate::capability::CommandDescriptor> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::Command(value) => Some(value.reference().descriptor()),
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Token(_)
            | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
        }
    }

    pub fn token_entry(&self) -> Option<&crate::capability::FrozenThemeTokenEntry> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::Token(value) => Some(value.entry()),
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
        }
    }

    pub fn resolved_token_entry(&self) -> Option<&crate::capability::FrozenThemeTokenEntry> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::Token(value) => {
                Some(value.semantics().resolved_target_entry())
            }
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::StateSlot(_) => None,
        }
    }

    pub fn state_slot_descriptor(&self) -> Option<&crate::capability::MosaicStateSlotDescriptor> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::StateSlot(value) => Some(value.descriptor()),
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::Token(_) => None,
        }
    }

    pub fn state_succession_is_launch(&self) -> Option<bool> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::StateSlot(value) => Some(matches!(
                value.succession(),
                crate::runtime::planning::execution_plan_input::WorthUiStateSlotSuccession::Launch
            )),
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::Token(_) => None,
        }
    }

    pub fn state_reconciliation_receipt(
        &self,
    ) -> Option<&crate::runtime::WorthUiDurableStateReconciliationReceipt> {
        match self.exact_meaning.as_deref()? {
            WorthUiPlanOrdinaryMeaning::StateSlot(value) => {
                match value.succession() {
                    crate::runtime::planning::execution_plan_input::WorthUiStateSlotSuccession::Launch => {
                        None
                    }
                    crate::runtime::planning::execution_plan_input::WorthUiStateSlotSuccession::Reconciled(
                        receipt,
                    ) => Some(receipt),
                }
            }
            WorthUiPlanOrdinaryMeaning::Component(_)
            | WorthUiPlanOrdinaryMeaning::Layout(_)
            | WorthUiPlanOrdinaryMeaning::ChildRange(_)
            | WorthUiPlanOrdinaryMeaning::Command(_)
            | WorthUiPlanOrdinaryMeaning::Token(_) => None,
        }
    }
}

impl WorthUiOrdinaryPlanSummaryRequest {
    #[cfg(any(test, feature = "certification-support"))]
    fn family(self) -> WorthUiPlanNodeInputFamily {
        match self {
            Self::Component => WorthUiPlanNodeInputFamily::ComponentInvocation,
            Self::ChildRange => WorthUiPlanNodeInputFamily::ChildRange,
            Self::Command => WorthUiPlanNodeInputFamily::Command,
            Self::Token => WorthUiPlanNodeInputFamily::TokenStyle,
            Self::StateSlot => WorthUiPlanNodeInputFamily::StateSlot,
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    fn target(self, handle: crate::runtime::WorthUiRuntimeHandle) -> WorthUiOrdinarySummaryTarget {
        match self {
            Self::Component => WorthUiOrdinarySummaryTarget::Component(
                WorthUiComponentHandle::from_runtime_handle(handle),
            ),
            Self::ChildRange => WorthUiOrdinarySummaryTarget::ChildRange(
                WorthUiChildRangeHandle::from_runtime_handle(handle),
            ),
            Self::Command => WorthUiOrdinarySummaryTarget::Command(
                WorthUiCommandHandle::from_runtime_handle(handle),
            ),
            Self::Token => {
                WorthUiOrdinarySummaryTarget::Token(WorthUiTokenHandle::from_runtime_handle(handle))
            }
            Self::StateSlot => WorthUiOrdinarySummaryTarget::StateSlot(
                WorthUiStateSlotHandle::from_runtime_handle(handle),
            ),
        }
    }
}
