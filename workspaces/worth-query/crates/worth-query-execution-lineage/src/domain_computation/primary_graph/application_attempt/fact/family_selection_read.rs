//! Attribution of a `current_output` family selection to its original attempt (public #429).
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use worth_relational::facade::identity::EntityId;

/// The key of one read's selection of `family` for `producer`. It names the
/// read, not its result: the issued observation is the reader's to hand over.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FamilySelectionRead {
    admission: WorthQueryOperationAdmissionIdentity,
    family: &'static str,
    producer: EntityId,
}

impl FamilySelectionRead {
    pub fn new(
        admission: WorthQueryOperationAdmissionIdentity,
        family: &'static str,
        producer: EntityId,
    ) -> Self {
        Self { admission, family, producer }
    }

    pub fn admits(&self, admission: WorthQueryOperationAdmissionIdentity) -> bool {
        self.admission == admission
    }
}
