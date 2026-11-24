//! The digested artifact payload: the transition bound to its sequence and
//! the commit position it follows. Emission borrows the positioned
//! transition; decoding owns the small record and grants nothing until
//! readmission.

use serde::{Deserialize, Serialize};
use worth_relational_branch::internal_port::branch::transition::positioned::{
    PositionedBranchTransition, PositionedBranchTransitionOwnerPort as _,
};

use worth_relational_branch::internal_port::branch::transition::record::RelationalBranchTransition;
use worth_relational_contracts::internal_port::branch::transition::sequence::RelationalBranchTransitionSequence;
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;

#[derive(Serialize)]
pub(super) struct TransitionPayloadRef<'a> {
    sequence: RelationalBranchTransitionSequence,
    follows: PatchStreamPosition,
    transition: &'a RelationalBranchTransition,
    schema_carrier: Option<[u8; 32]>,
    caller_association: Option<&'a worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
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
    pub(super) caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
}

fn required_schema_carrier<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<[u8; 32]>, D::Error> {
    Option::deserialize(deserializer)
}

fn required_association<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>, D::Error>{
    Option::deserialize(deserializer)
}
