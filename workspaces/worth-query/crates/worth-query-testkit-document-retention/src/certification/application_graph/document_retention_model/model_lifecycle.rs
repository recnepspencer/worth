//! Ordinary document retirement/recreation against the surviving related model.
#[path = "model_lifecycle/declaration.rs"]
pub mod declaration;
#[path = "model_lifecycle/handler.rs"]
pub mod handler;
#[path = "model_lifecycle/program.rs"]
pub mod program;
pub use declaration::{declare, ModelLifecycleBinding, ModelLifecycleInput, ModelLifecycleIntent};
pub use handler::ModelLifecycleHandler;
pub use program::{validated_model_lifecycle_program, ModelLifecycleProgram};
