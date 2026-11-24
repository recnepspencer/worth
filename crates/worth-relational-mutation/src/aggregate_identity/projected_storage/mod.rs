//! The additional private grammar used only by projected aggregate bindings.
pub mod binding;
pub mod case_roster;
pub mod domain_text;

pub(crate) use crate::contracts::schema::aggregate_identity::projected_grammar::{
    COORDINATE_ASPECT, COORDINATE_FIELDS, MEMBERSHIP_ASPECT,
};
pub(super) use binding::{domain_for_contract, member_key};
pub(super) use case_roster::{decode_cases, encode_cases};
pub(super) use domain_text::{decode_domain, encode_domain};
