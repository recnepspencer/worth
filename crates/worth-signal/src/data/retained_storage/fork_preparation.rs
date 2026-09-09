use super::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial as Denial,
};

/// Representation facts for one persistent fork, never resource admission.
/// The source keeps its conversion storage even after the returned fork drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetainedStorageForkCharge {
    pub(crate) retained: Charge,
    pub(crate) source_growth: Charge,
}

impl RetainedStorageForkCharge {
    pub(crate) fn from_charges(source: Charge, retained: Charge) -> Result<Self, Denial> {
        Ok(Self {
            retained,
            source_growth: retained.checked_sub(source)?,
        })
    }

    pub(crate) fn unchanged(retained: Charge) -> Self {
        Self {
            retained,
            source_growth: Charge::ZERO,
        }
    }

    pub(crate) fn checked_add(self, other: Self) -> Result<Self, Denial> {
        Ok(Self {
            retained: self.retained.checked_add(other.retained)?,
            source_growth: self.source_growth.checked_add(other.source_growth)?,
        })
    }
}

/// Explicit bounded preparation before constructing retained roots. Implementors
/// may install charge facts but must not allocate or convert the representation.
pub(crate) trait RetainedStorageForkPreparation {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial>;
}
