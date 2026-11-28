//! Compile-time contract doctests for [`WorthQueryHostSession`],
//! [`WorthQueryTypedLiveSubscription`], [`WorthQueryPreviewFork`],
//! [`WorthQueryCorrectionEntry`].
//!
//! Persisted correction entries, open dispositions, and close denials satisfy
//! the thread-safety and unwind-safety contracts for desktop UI state.
//!
//! ```
//! use std::panic::{RefUnwindSafe, UnwindSafe};
//! use worth_query_host::facade::{
//!     declaration::{
//!         application_program::ApplicationProgramDefinition,
//!     },
//!     domain::ApplicationSchema,
//!     WorthQueryApplicationHome, WorthQueryCorrectionEntry, WorthQueryHomeOpenDisposition,
//!     WorthQueryHostSession, WorthQueryTypedLiveCloseDenial,
//!     DEFAULT_HOST_EXECUTION_POLICY, DEFAULT_HOST_EXECUTION_WORK_CEILING,
//! };
//!
//! fn assert_static_and_safe<T: Send + Sync + UnwindSafe + RefUnwindSafe + 'static>() {}
//! assert_static_and_safe::<WorthQueryCorrectionEntry>();
//! assert_static_and_safe::<WorthQueryHomeOpenDisposition>();
//! assert_static_and_safe::<WorthQueryTypedLiveCloseDenial>();
//! assert!(DEFAULT_HOST_EXECUTION_WORK_CEILING > 0);
//! assert!(DEFAULT_HOST_EXECUTION_WORK_CEILING < u64::MAX);
//! let _ = DEFAULT_HOST_EXECUTION_POLICY;
//!
//! fn session_can_borrow_or_own_principal<'home, S, P>(
//!     home: &'home WorthQueryApplicationHome<S, P>,
//!     me: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S>,
//! ) -> WorthQueryHostSession<'home, S, P>
//! where
//!     S: ApplicationSchema,
//!     P: ApplicationProgramDefinition<S>,
//! {
//!     home.session(me).with_policy(DEFAULT_HOST_EXECUTION_POLICY)
//! }
//! ```
//!
//! A preview fork cannot be silently duplicated; adoption or discard consumes it:
//!
//! ```compile_fail,E0382
//! use worth_query_host::facade::{domain::ApplicationSchema, WorthQueryPreviewFork};
//! fn duplicate_preview<S: ApplicationSchema>(preview: WorthQueryPreviewFork<'_, S>) {
//!     let first = preview;
//!     let second = preview;
//!     drop((first, second));
//! }
//! ```
//!
//! A live subscription cannot be duplicated; closing it consumes the handle:
//!
//! ```compile_fail,E0382
//! use worth_query_host::facade::{
//!     declaration::{
//!         application_query::{ApplicationQueryBinding, ApplicationQueryIntent},
//!         application_schema::ApplicationStructuredValueBinding,
//!     },
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryApplicationProjection,
//!     WorthQueryTypedLiveSubscription,
//! };
//! fn duplicate_live<S, I>(sub: WorthQueryTypedLiveSubscription<'_, S, I>)
//! where
//!     S: ApplicationSchema,
//!     I: ApplicationQueryIntent<S>,
//!     <<I::Binding as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
//!         WorthQueryApplicationProjection<S, <I::Binding as ApplicationQueryBinding<S>>::Query>,
//! {
//!     let first = sub;
//!     let second = sub;
//!     drop((first, second));
//! }
//! ```
