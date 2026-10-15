//! Native-backed input construction is closed to both normal consumer audiences.
//! ```compile_fail,E0599
//! use worth_query_host::facade::declaration::application_operation::ApplicationEncodedInput;
//! use worth_query_decl::facade::application_schema::ApplicationStructuredValueBinding;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! struct Switch { a:u64, b:u64, reads:AtomicUsize }
//! impl Switch {
//!     fn value(&self)->&u64 { if self.reads.fetch_add(1,Ordering::SeqCst)==0 { &self.a } else { &self.b } }
//! }
//! fn cannot_encode_a_then_lend_b<B:ApplicationStructuredValueBinding<Value=u64>>() {
//!     let switch=Switch {a:1,b:2,reads:AtomicUsize::new(0)};
//!     assert_eq!(*switch.value(),1);
//!     let encoded=ApplicationEncodedInput::<B>::encode_owned(switch).unwrap();
//!     let _:worth_query_decl::facade::application_operation::ApplicationEncodedInput<B>=encoded;
//! }
//! ```
//! Plain owned values remain the consumer construction route, with no custom backer.
//! ```
//! use worth_query_host::facade::declaration::application_operation::ApplicationEncodedInput;
//! use worth_query_decl::facade::application_schema::{ApplicationStructuredValueBinding,ApplicationValueEncodeDenial};
//! fn owned_value<B:ApplicationStructuredValueBinding<Value=u64>>() -> Result<ApplicationEncodedInput<B>,ApplicationValueEncodeDenial> {
//!     ApplicationEncodedInput::<B>::encode(1)
//! }
//! ```
