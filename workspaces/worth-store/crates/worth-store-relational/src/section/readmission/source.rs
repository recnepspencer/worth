//! Mutually exclusive actual Native source DATA; no strategy or phase marker.
use worth_relational::facade::durability::{
    FreshRelationalSourceAdmission, RecoveredRelationalRuntimeAuthority, RecoveryOutcome,
};

pub(in crate::section) enum ReadmittedRelationalSource {
    Fresh(FreshRelationalSourceAdmission),
    Recovered {
        authority: RecoveredRelationalRuntimeAuthority,
        outcome: RecoveryOutcome,
    },
}
