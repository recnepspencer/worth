//! Fresh admission preserves the original initialization publisher's fate.
use crate::durability::published_medium::RelationalInitialInputsPublicationStop;
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;

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
