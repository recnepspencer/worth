//! Native capability admission and lifecycle commands preserve source publication custody.
#[path = "capability/grant_commands.rs"]
mod grant_commands;
#[path = "capability/grants.rs"]
mod grants;
#[path = "capability/handler.rs"]
mod handler;
#[path = "capability/host.rs"]
mod host;
#[path = "capability/interleaving.rs"]
mod interleaving;
#[path = "capability/program.rs"]
mod program;
#[path = "capability/readback.rs"]
mod readback;
#[path = "capability/recovery.rs"]
mod recovery;
#[path = "capability/schema.rs"]
mod schema;
