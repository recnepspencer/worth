mod child_range;
mod command;
mod component;
mod digest;
mod layout;
mod state_slot;
mod token;

pub(crate) use child_range::WorthUiChildRangePlanMeaning;
pub(crate) use command::WorthUiCommandPlanMeaning;
pub(crate) use component::WorthUiComponentPlanMeaning;
pub(crate) use layout::WorthUiLayoutPlanMeaning;
#[cfg(test)]
pub(crate) use state_slot::WorthUiStateSlotMeaningDenial;
pub(crate) use state_slot::{
    durable_family_for_slot, WorthUiStateSlotPlanMeaning, WorthUiStateSlotSuccession,
};
pub(crate) use token::WorthUiTokenPlanMeaning;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiPlanOrdinaryMeaning {
    Component(WorthUiComponentPlanMeaning),
    Layout(WorthUiLayoutPlanMeaning),
    ChildRange(WorthUiChildRangePlanMeaning),
    Command(Box<WorthUiCommandPlanMeaning>),
    Token(WorthUiTokenPlanMeaning),
    StateSlot(WorthUiStateSlotPlanMeaning),
}

impl WorthUiPlanOrdinaryMeaning {
    pub(crate) fn family(&self) -> crate::runtime::WorthUiPlanNodeInputFamily {
        match self {
            Self::Component(_) => crate::runtime::WorthUiPlanNodeInputFamily::ComponentInvocation,
            Self::Layout(_) => crate::runtime::WorthUiPlanNodeInputFamily::LayoutRegion,
            Self::ChildRange(_) => crate::runtime::WorthUiPlanNodeInputFamily::ChildRange,
            Self::Command(_) => crate::runtime::WorthUiPlanNodeInputFamily::Command,
            Self::Token(_) => crate::runtime::WorthUiPlanNodeInputFamily::TokenStyle,
            Self::StateSlot(_) => crate::runtime::WorthUiPlanNodeInputFamily::StateSlot,
        }
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        match self {
            Self::Component(value) => value.semantic_digest(),
            Self::Layout(value) => value.semantic_digest(),
            Self::ChildRange(value) => value.semantic_digest(),
            Self::Command(value) => value.semantic_digest(),
            Self::Token(value) => value.semantic_digest(),
            Self::StateSlot(value) => value.semantic_digest(),
        }
    }

    pub(crate) fn child_range_identity(&self) -> Option<&str> {
        match self {
            Self::Component(value) => value.child_range_identity(),
            Self::Layout(value) => value.child_range_identity(),
            _ => None,
        }
    }

    pub(crate) fn child_range(&self) -> Option<&WorthUiChildRangePlanMeaning> {
        match self {
            Self::ChildRange(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn dependency_identities(&self) -> Vec<&str> {
        if let Some(range) = self.child_range() {
            return range
                .child_identities()
                .iter()
                .map(String::as_str)
                .collect();
        }
        self.child_range_identity().into_iter().collect()
    }

    pub(crate) fn same_mounted_layout_meaning(&self, successor: &Self) -> bool {
        match (self, successor) {
            (Self::Component(previous), Self::Component(successor)) => {
                previous.same_mounted_layout_meaning(successor)
            }
            (Self::Layout(previous), Self::Layout(successor)) => {
                previous.same_mounted_layout_meaning(successor)
            }
            (Self::ChildRange(_), Self::ChildRange(_)) => true,
            (Self::StateSlot(previous), Self::StateSlot(successor)) => {
                previous.same_mounted_layout_meaning(successor)
            }
            _ => self == successor,
        }
    }
}
