//! Receipt metadata remains available through the native audience facade.
//! ```
//! use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn metadata(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.commit_reference();
//!     let _ = receipt.committed_product_publication();
//!     let _ = receipt.has_retained_preimage();
//! }
//! ```
//! Raw pre-images require a freshly disclosed owner path.
//! ```compile_fail,E0624
//! use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn payload(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.retained_preimage();
//! }
//! ```
//! A receipt's historical authority binding cannot disclose its original input.
//! ```compile_fail,E0624
//! use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn payload<T: 'static>(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.authority_binding().retained_governed_input::<T>();
//! }
//! ```
