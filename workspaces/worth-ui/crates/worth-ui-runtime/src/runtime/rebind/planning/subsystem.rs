use crate::graph::UiGraphFactConsumerKey;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiRebindSubsystemKind {
    Preservation,
    Graph,
    Mount,
    Measurement,
    Allocation,
    Binding,
    Obligation,
    Surface,
    Retirement,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiRebindPlanTarget {
    Consumer(UiGraphFactConsumerKey),
    QueryBinding(Box<str>),
    Surface(worth_ui_host_contract::UiSemanticSurfaceIdentity),
}

#[derive(Debug)]
pub struct UiRebindSubsystemPlan {
    kind: UiRebindSubsystemKind,
    targets: Box<[UiRebindPlanTarget]>,
}

impl UiRebindSubsystemPlan {
    pub(crate) fn new(kind: UiRebindSubsystemKind, mut targets: Vec<UiRebindPlanTarget>) -> Self {
        targets.sort();
        targets.dedup();
        Self {
            kind,
            targets: targets.into_boxed_slice(),
        }
    }

    pub const fn kind(&self) -> UiRebindSubsystemKind {
        self.kind
    }

    pub fn targets(&self) -> &[UiRebindPlanTarget] {
        &self.targets
    }
}
