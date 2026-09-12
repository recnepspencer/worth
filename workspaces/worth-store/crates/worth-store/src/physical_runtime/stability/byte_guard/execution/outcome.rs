use core::convert::Infallible;

use worth_proof::TransitionOutcome;

use super::{EpochRetryReceipt, PhysicalReadExecutionDenial, StablePhysicalReadReceipt};

pub type StablePhysicalReadExecutionOutcome = TransitionOutcome<
    StablePhysicalReadReceipt,
    PhysicalReadExecutionDenial,
    Infallible,
    EpochRetryReceipt,
    EpochRetryReceipt,
    Infallible,
>;

pub type StablePhysicalReadEpochFreshnessOutcome = TransitionOutcome<
    (),
    PhysicalReadExecutionDenial,
    Infallible,
    EpochRetryReceipt,
    EpochRetryReceipt,
    Infallible,
>;
