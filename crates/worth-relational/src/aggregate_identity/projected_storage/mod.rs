//! The additional private grammar used only by projected aggregate bindings.
mod binding;
mod case_roster;
mod domain_text;

pub(crate) use crate::schema::aggregate_identity::projected_grammar::{
    COORDINATE_ASPECT, COORDINATE_FIELDS, MEMBERSHIP_ASPECT,
};
pub(super) use binding::{domain_for_contract, member_key};
pub(super) use case_roster::{decode_cases, encode_cases};
pub(super) use domain_text::{decode_domain, encode_domain};
