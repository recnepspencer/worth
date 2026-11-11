//! Fresh admission preserves the original initialization publisher's fate.
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::medium::RelationalInitialInputsPublicationStop;

#[derive(Debug)]
pub enum RelationalFreshSourceAdmissionStop {
    Recovery(RelationalRecoveryStop),
    Publication(RelationalInitialInputsPublicationStop),
}

impl From<RelationalRecoveryStop> for RelationalFreshSourceAdmissionStop {
    fn from(cause: RelationalRecoveryStop) -> Self {
        Self::Recovery(cause)
    }
}
