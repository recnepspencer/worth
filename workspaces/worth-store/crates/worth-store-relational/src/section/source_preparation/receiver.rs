//! A compiler preparation is consumed once; it never falls back to replanning.
use worth_relational::facade::durability::PreparedRelationalHomeRecovery;

pub(in crate::section) enum HomeReceiverPreparation {
    NotRequested,
    Prepared(PreparedRelationalHomeRecovery),
    Consumed,
}
