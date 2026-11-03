/// A Query outcome row's exact reference into its enclosing publication.
/// The row's immutable creation envelope supplies the original association;
/// a slot reference supplies no live settlement or output-delivery authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableQueryOutputSettlement {
    Absent,
    OriginalOccurrenceSlot { slot: u64 },
}

impl DurableQueryOutputSettlement {
    pub const fn recorded_code(self) -> Option<u64> {
        match self {
            Self::Absent => Some(0),
            Self::OriginalOccurrenceSlot { slot } => slot.checked_add(1),
        }
    }

    pub const fn from_recorded_code(code: u64) -> Self {
        match code {
            0 => Self::Absent,
            value => Self::OriginalOccurrenceSlot { slot: value - 1 },
        }
    }
}
