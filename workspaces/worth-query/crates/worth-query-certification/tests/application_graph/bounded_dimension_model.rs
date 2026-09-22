//! A public consumer's bounded-dimension application.
//!
//! One schema installs two commit-boundary rule contracts with distinct stable
//! identifiers. Program P0 declares `bounded-dimension-v1`, program P1 declares
//! `bounded-dimension-v2`, and nothing else about them differs. Everything here
//! is authored through the `worth-query-host` and `worth-query-decl` facades,
//! exactly as an external consumer would have to author it.

#[path = "bounded_dimension_model/dimension_entry.rs"]
pub mod dimension_entry;
#[path = "bounded_dimension_model/host.rs"]
pub mod host;
#[path = "bounded_dimension_model/operator_identity.rs"]
pub mod operator_identity;
#[path = "bounded_dimension_model/presented_request.rs"]
pub mod presented_request;
#[path = "bounded_dimension_model/programs.rs"]
pub mod programs;
#[path = "bounded_dimension_model/readback.rs"]
pub mod readback;
#[path = "bounded_dimension_model/rules.rs"]
pub mod rules;
#[path = "bounded_dimension_model/schema.rs"]
pub mod schema;
#[path = "bounded_dimension_model/settled_verdict.rs"]
pub mod settled_verdict;
