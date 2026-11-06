//! Code-generation workload: four applications share forty aspect declarations.
#![recursion_limit = "512"]
mod model;
use model::*;
use worth_query_declaration::facade::application_schema::{fragment::*, *};
use worth_query_execution::facade::primary_graph::*;

mod application_0;
pub use application_0::application_0;
mod application_1;
pub use application_1::application_1;
mod application_2;
pub use application_2::application_2;
mod application_3;
pub use application_3::application_3;

mod extensions;
pub use extensions::extension_views;
pub use extensions::{
    extension_application_0, extension_application_1, extension_application_2,
    extension_application_3,
};
