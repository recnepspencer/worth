//! The whole reconstruction's decode owner vetoes atomic view publication.
use super::Denial;
use crate::domain_computation::primary_graph::{
    application_query::one_shot::outcome::value_decode_denial, value_decode::ValueDecodeState,
};
use worth_relational::facade::identity::EntityId;

pub(super) fn finish(
    decode: &ValueDecodeState,
    last_decoded_root: Option<EntityId>,
) -> Result<(), Denial> {
    match decode.denial() {
        None => Ok(()),
        Some(cause) => Err(Denial::ReadDenied {
            root: last_decoded_root.expect("a decode refusal belongs to a staged member"),
            denial: value_decode_denial(cause, "paired-reconstruction"),
        }),
    }
}
