//! One native existing-reference action, its independent query and boundary proofs.
#[path = "reference_input/admission.rs"]
pub mod admission;
#[path = "reference_input/signature_evidence.rs"]
pub mod signature_evidence;
#[path = "reference_input/signature_query.rs"]
pub mod signature_query;
#[path = "reference_input/signing.rs"]
pub mod signing;
pub use signing::SignBinding;
#[path = "reference_input/declaration.rs"]
pub mod declaration;
#[path = "reference_input/evidence.rs"]
pub mod evidence;
#[path = "reference_input/query.rs"]
pub mod query;
pub use declaration::{declare, Assign, Binding};
