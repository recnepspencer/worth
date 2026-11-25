//! Producer stop origins through the application audience facade.
//!
//! Producer publication and admission diagnostics borrow the returned denial;
//! the application keeps the original terminal operation and its cleanup custody.
//! Native companion amounts are data, not a new retry or publication authority.
//!
//! ```
//! use worth_query_host::facade::{
//!     application_entry::{
//!         WorthQueryApplicationOutputDemandDenial,
//!         WorthQueryOperationAdvanceDenial,
//!         WorthQueryRequiredOutputPreparationDenial,
//!     },
//!     primary_graph::{CompanionPreflightStop, VerificationCapacityStop},
//! };
//! fn producer_stop_origins(
//!     denial: &WorthQueryOperationAdvanceDenial,
//! ) -> (Option<CompanionPreflightStop>, Option<CompanionPreflightStop>, Option<VerificationCapacityStop>) {
//!     let WorthQueryOperationAdvanceDenial::Outputs(required) = denial else {
//!         return (None, None, None);
//!     };
//!     let native = match required {
//!         WorthQueryRequiredOutputPreparationDenial::DemandExecution(native) => native,
//!         WorthQueryRequiredOutputPreparationDenial::Demand(
//!             WorthQueryApplicationOutputDemandDenial::Demand(native),
//!         ) => native,
//!         _ => return (None, None, None),
//!     };
//!     (native.publication_companion_stop(), native.producer_admission_stop(),
//!      native.output_verification_capacity_stop())
//! }
//! ```
