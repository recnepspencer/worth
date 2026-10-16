//! Owning complete no-source trees retain no construction request lifetime.
//!
//! ```
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryProgramApplicationRuntime};
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn retain<S,P,R>(operation: WorthQueryProgramOutputOperation<S,P,R>) -> WorthQueryProgramOutputOperation<S,P,R>
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { operation }
//! fn retain_refusal<S,P,R>(refusal: WorthQueryProgramOutputConstructionRefusal<S,P,R>) -> WorthQueryProgramOutputConstructionRefusal<S,P,R>
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { refusal }
//! ```
//!
//! Pending tree custody cannot publish a current read.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryProgramApplicationRuntime};
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn pending_read<S,P,R>(operation: WorthQueryProgramOutputOperation<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { let _ = operation.read_current(); }
//! ```
//!
//! An operation does not expose its retained runtime.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryProgramApplicationRuntime};
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn runtime<S,P,R>(operation: WorthQueryProgramOutputOperation<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { let _ = operation.runtime(); }
//! ```
//!
//! Native tree obligations are move-only.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryProgramApplicationRuntime};
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn clone_owner<S,P,R>(operation: WorthQueryProgramOutputOperation<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { let _ = operation.clone(); }
//! ```
//!
//! A terminal request cannot rerun a producer.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryProgramApplicationRuntime};
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn terminal_advance<S,P,R>(operation: WorthQueryTerminalProgramOutputOperation<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { let _ = operation.advance(); }
//! ```
//!
//! A performed native tree cannot acquire the read-origin ordinary close contract.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryProgramApplicationRuntime};
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn native_conversion<S,P,R>(operation: WorthQueryApplicationProgramOutputHandle<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { let _ = operation.into_no_source_operation(); }
//! ```
//!
//! A refused close retains its released prefix and cannot restart production.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn closing_advance<S,P,R>(operation: WorthQueryProgramOutputCloseRefusal<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> { let _ = operation.advance(); }
//! ```
