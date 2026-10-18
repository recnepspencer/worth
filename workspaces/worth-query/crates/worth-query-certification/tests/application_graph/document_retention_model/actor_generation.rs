//! Real principal retirement/rebirth operations for the actor identity court.
//! Workflow actor records are never seeded or edited here.
#[path = "actor_generation/binding.rs"]
mod binding;
#[path = "actor_generation/declaration.rs"]
mod declaration;
#[path = "actor_generation/handler.rs"]
mod handler;
#[path = "actor_generation/program.rs"]
mod program;
pub use binding::{ActorLifecycleBinding, ActorLifecycleIntent};
pub use declaration::{declare, ActorLifecycleInput};
pub use handler::ActorLifecycleHandler;
pub use program::{validated_actor_program, ActorProgram};
