use super::super::UiObservationFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiObservationTurnDenial {
    TurnAlreadyActive,
    IdentityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiObservationAdmissionDenial {
    ForeignSession,
    ForeignSourceBasis,
    ForeignQueryProjection,
    DuplicateOwnerOrder,
    HistoricalOwnerOrder,
    DuplicateFamily,
    TurnCapacityExceeded,
    ByteCapacityExceeded,
    PoisonedTurn,
    EmptyTurn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiObservationAdmissionReceipt {
    family: UiObservationFamily,
    owner_order: u64,
    retained_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiObservationSetSummary {
    admitted_count: usize,
    retained_bytes: usize,
    families: Box<[UiObservationFamily]>,
}

#[derive(Debug)]
pub enum UiQueryObservationAdmissionStop {
    Observation(UiObservationAdmissionDenial),
    Query(Box<worth_ui_query_binding::WorthUiCollectionChangeAdmissionStop>),
}

impl UiObservationAdmissionReceipt {
    pub(super) const fn new(
        family: UiObservationFamily,
        owner_order: u64,
        retained_bytes: usize,
    ) -> Self {
        Self {
            family,
            owner_order,
            retained_bytes,
        }
    }

    pub const fn family(self) -> UiObservationFamily {
        self.family
    }

    pub const fn owner_order(self) -> u64 {
        self.owner_order
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}

impl UiObservationSetSummary {
    pub(super) fn new(
        admitted_count: usize,
        retained_bytes: usize,
        families: Box<[UiObservationFamily]>,
    ) -> Self {
        Self {
            admitted_count,
            retained_bytes,
            families,
        }
    }

    pub const fn admitted_count(&self) -> usize {
        self.admitted_count
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub fn families(&self) -> &[UiObservationFamily] {
        &self.families
    }
}
