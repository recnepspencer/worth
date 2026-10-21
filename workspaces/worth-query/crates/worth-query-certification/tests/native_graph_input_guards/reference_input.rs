//! One native existing-reference action, its independent query and boundary proofs.
#[path = "reference_input/admission.rs"]
mod admission;
#[path = "reference_input/signature_evidence.rs"]
mod signature_evidence;
#[path = "reference_input/signature_query.rs"]
mod signature_query;
#[path = "reference_input/signing.rs"]
mod signing;
pub use signing::SignBinding;
#[path = "reference_input/declaration.rs"]
mod declaration;
#[path = "reference_input/evidence.rs"]
mod evidence;
#[path = "reference_input/query.rs"]
mod query;
pub use declaration::{declare, Assign, Binding};
