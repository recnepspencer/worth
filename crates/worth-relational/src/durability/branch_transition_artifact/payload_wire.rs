//! The digested artifact payload: the transition bound to its sequence and
//! the commit position it follows. Emission borrows the positioned
//! transition; decoding owns the small record and grants nothing until
//! readmission.

use serde::{Deserialize, Serialize};

use crate::branch::{
    PositionedBranchTransition, RelationalBranchTransition, RelationalBranchTransitionSequence,
};
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Serialize)]
pub(super) struct TransitionPayloadRef<'a> {
    sequence: RelationalBranchTransitionSequence,
    follows: PatchStreamPosition,
    transition: &'a RelationalBranchTransition,
}

impl<'a> TransitionPayloadRef<'a> {
    pub(super) fn new(positioned: &'a PositionedBranchTransition) -> Self {
        Self {
            sequence: positioned.sequence(),
            follows: positioned.follows(),
            transition: positioned.transition(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TransitionPayloadWire {
    pub(super) sequence: RelationalBranchTransitionSequence,
    pub(super) follows: PatchStreamPosition,
    pub(super) transition: RelationalBranchTransition,
}
