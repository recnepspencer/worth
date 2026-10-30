//! Signed parent bookkeeping uses the ordinary native candidate and authentication owners.
#[path = "path_signing/admission.rs"]
pub mod admission;
#[path = "path_signing/authentication.rs"]
pub mod authentication;
#[path = "path_signing/binding.rs"]
pub mod binding;
#[path = "path_signing/declaration.rs"]
pub mod declaration;
#[path = "path_signing/evidence.rs"]
pub mod evidence;
#[path = "path_signing/query.rs"]
pub mod query;
#[path = "path_signing/schema.rs"]
pub mod schema;
pub use binding::{GraphAliasBinding, GraphBinding, ScopeAliasBinding};
pub use declaration::{declare, SignGraph, SignGraphAlias, SignScopeAlias};
