//! Native capability admission and lifecycle commands preserve source publication custody.
#[path = "capability/authorization.rs"]
pub mod authorization;
#[path = "capability/grant_commands.rs"]
pub mod grant_commands;
#[path = "capability/grants.rs"]
pub mod grants;
#[path = "capability/handler.rs"]
pub mod handler;
#[path = "capability/host.rs"]
pub mod host;
#[path = "capability/interleaving.rs"]
pub mod interleaving;
#[path = "capability/program.rs"]
pub mod program;
#[path = "capability/readback.rs"]
pub mod readback;
#[path = "capability/schema.rs"]
pub mod schema;

#[path = "capability/resources.rs"]
pub mod resources;
#[path = "capability/workflow.rs"]
pub mod workflow;
