mod consequence_topology;
mod facts;
mod ia_04;
mod intent_types;
mod model;
mod portal_topology;
mod route_scale_world;
mod topology;
mod world;

pub(in crate::intent) use consequence_topology::{
    build_consequence_with_provider, build_consequence_with_provider_and_profile,
    consequence_replacement_input,
};
pub(in crate::intent) use facts::OperabilityFacts;
pub(in crate::intent) use intent_types::{
    ConsequenceIntent, ConsequenceOutcome, EmptyOutcome, PrimaryIntent,
};
pub(in crate::intent) use portal_topology::{
    build_open_portal_application, build_open_portal_application_with_host,
    build_open_portal_projection_application_with_host,
    build_open_portal_two_focus_application_with_host, portal_owner_removal_input,
};
pub(in crate::intent) use route_scale_world::{last_route_graph_node, MountedRouteScaleWorld};
pub(in crate::intent) use topology::{
    build_route_scale, build_scoped, build_scoped_with_provider,
    build_scoped_with_provider_observation, replacement_input, OccupancyLayout,
};
