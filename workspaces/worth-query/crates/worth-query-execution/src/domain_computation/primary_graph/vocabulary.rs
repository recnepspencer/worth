//! Each family's vocabulary: the refusals, observations, identities and evidence its
//! operations exchange, held below every family so one family can name another's without a
//! cycle. Each child holds one family's items, moved whole from that family.
pub(crate) mod aggregate_observation;
pub(crate) mod application_attempt;
pub(crate) mod application_contribution;
pub(crate) mod application_inbound;
pub(crate) mod application_installation;
pub(crate) mod application_invariant;
pub(crate) mod application_output_demand;
pub(crate) mod application_query;
pub(crate) mod application_runtime;
pub(crate) mod bootstrap;
pub(crate) mod component_selection;
pub(crate) mod conditional_operation;
pub(crate) mod entity_resolution_denial;
pub(crate) mod exact_basis_access;
pub(crate) mod freshness;
pub(crate) mod handler;
pub(crate) mod index_currency;
pub(crate) mod invariant_projection;
pub(crate) mod managed_bridge;
pub(crate) mod operation_body;
pub(crate) mod output_binding_identity;
pub(crate) mod output_family_identity;
pub(crate) mod output_lineage;
pub(crate) mod product_activation;
pub(crate) mod product_operation;
pub(crate) mod provider;
pub(crate) mod resolution_denial;
pub(crate) mod structural_walk;
pub(crate) mod value_decode;
