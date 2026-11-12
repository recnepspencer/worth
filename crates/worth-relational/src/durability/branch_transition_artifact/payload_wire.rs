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
    schema_carrier: Option<[u8; 32]>,
    caller_association: Option<&'a crate::history::data::RelationalCallerAssociation>,
}

impl<'a> TransitionPayloadRef<'a> {
    pub(super) fn new(positioned: &'a PositionedBranchTransition) -> Self {
        Self {
            sequence: positioned.sequence(),
            follows: positioned.follows(),
            transition: positioned.transition(),
            schema_carrier: positioned.schema_carrier(),
            caller_association: positioned.caller_association(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TransitionPayloadWire {
    pub(super) sequence: RelationalBranchTransitionSequence,
    pub(super) follows: PatchStreamPosition,
    pub(super) transition: RelationalBranchTransition,
    #[serde(deserialize_with = "required_schema_carrier")]
    pub(super) schema_carrier: Option<[u8; 32]>,
    #[serde(deserialize_with = "required_association")]
    pub(super) caller_association: Option<crate::history::data::RelationalCallerAssociation>,
}

fn required_schema_carrier<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<[u8; 32]>, D::Error> {
    Option::deserialize(deserializer)
}

fn required_association<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::history::data::RelationalCallerAssociation>, D::Error> {
    Option::deserialize(deserializer)
}
