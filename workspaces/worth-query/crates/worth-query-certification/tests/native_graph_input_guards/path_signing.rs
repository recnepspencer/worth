//! Signed parent bookkeeping uses the ordinary native candidate and authentication owners.
#[path = "path_signing/admission.rs"]
mod admission;
#[path = "path_signing/authentication.rs"]
pub(super) mod authentication;
#[path = "path_signing/binding.rs"]
mod binding;
#[path = "path_signing/declaration.rs"]
mod declaration;
#[path = "path_signing/evidence.rs"]
mod evidence;
#[path = "path_signing/query.rs"]
mod query;
#[path = "path_signing/schema.rs"]
mod schema;
pub use binding::{GraphAliasBinding, GraphBinding, ScopeAliasBinding};
pub use declaration::{declare, SignGraph, SignGraphAlias, SignScopeAlias};
