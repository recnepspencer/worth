//! Correction alternatives carry historical evidence; only Query can construct
//! their selections. The public `application_correction` example exercises
//! selection, retained input preparation and ordinary publication together.
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::application_entry::WorthQuerySelectedCorrectionAlternative;
//! fn forge<'a, S>() -> WorthQuerySelectedCorrectionAlternative<'a, S> {
//!     WorthQuerySelectedCorrectionAlternative {
//!         source: todo!(), root: todo!(), selected: todo!(), active: None,
//!         _selected_ancestry: todo!(), _active_ancestry: None,
//!     }
//! }
//! ```
//!
//! An applicability observation cannot replace the selected alternative.
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{domain::ApplicationSchema, application_entry::{
//!     WorthQueryApplicationCorrections, WorthQueryCorrectionApplicability,
//! }};
//! fn report_is_not_selection<S: ApplicationSchema>(
//!     request: WorthQueryApplicationCorrections<'_, '_, '_, S>,
//!     report: WorthQueryCorrectionApplicability,
//! ) { let _ = request.reapply(&report); }
//! ```
//!
//! Publication consumes its preparation.
//!
//! ```compile_fail,E0382
//! use worth_query_host::facade::application_entry::WorthQueryPreparedApplicationCorrection;
//! fn publish_twice(prepared: WorthQueryPreparedApplicationCorrection<'_>) {
//!     let _ = prepared.publish();
//!     let _ = prepared.publish();
//! }
//! ```
//!
//! Reapplication derives its input from retained source custody.
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{domain::ApplicationSchema,
//!     application_entry::WorthQueryApplicationReapplicationRequest};
//! fn replace_input<S: ApplicationSchema, Input>(
//!     request: WorthQueryApplicationReapplicationRequest<'_, '_, '_, '_, S>,
//!     replacement: Input,
//! ) { let _ = request.input(replacement); }
//! ```
//!
//! The retained pre-image exposes immutable values.
//!
//! ```compile_fail,E0594
//! use worth_query_host::facade::declaration::application_aftermath::{
//!     ApplicationCorrectionPreImage, DeclaredPreImageLocus,
//! };
//! fn mutate_retained<S>(
//!     read: &dyn ApplicationCorrectionPreImage<S>, locus: &DeclaredPreImageLocus<S>,
//! ) { let value = read.field(locus).unwrap(); *value = (*value).clone(); }
//! ```
//!
//! The lawful counterparts select from a disclosed page, prepare retained input,
//! publish once and observe the original value through the read-only view.
//!
//! ```
//! use worth_query_host::facade::{domain::ApplicationSchema, application_entry::{
//!     WorthQueryApplicationCorrectionAlternatives, WorthQueryCorrectionAlternativeRef,
//!     WorthQueryCorrectionAlternativesPage,
//!     WorthQueryCorrectionHistoryLimits, WorthQuerySelectedCorrectionAlternative,
//!     WorthQueryApplicationReapplicationRequest, WorthQueryApplicationCorrectionPreparation,
//!     WorthQueryApplicationCorrectionDenial, WorthQueryPreparedApplicationCorrection,
//!     WorthQueryApplicationCorrectionOutcome,
//! }, declaration::application_aftermath::{ApplicationCorrectionPreImage, DeclaredPreImageLocus}};
//! fn select<'a, S: ApplicationSchema>(
//!     page: &WorthQueryApplicationCorrectionAlternatives<'a, '_, '_, S>,
//!     alternative: &WorthQueryCorrectionAlternativeRef, limits: WorthQueryCorrectionHistoryLimits,
//! ) -> Result<WorthQuerySelectedCorrectionAlternative<'a, S>, WorthQueryApplicationCorrectionDenial> {
//!     page.select(alternative, limits)
//! }
//! fn detach<S: ApplicationSchema>(
//!     page: WorthQueryApplicationCorrectionAlternatives<'_, '_, '_, S>,
//! ) -> WorthQueryCorrectionAlternativesPage { page.into_page() }
//! fn prepare<'a, S: ApplicationSchema>(
//!     request: WorthQueryApplicationReapplicationRequest<'a, '_, '_, '_, S>,
//! ) -> Result<WorthQueryApplicationCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial> {
//!     request.prepare()
//! }
//! fn publish(prepared: WorthQueryPreparedApplicationCorrection<'_>)
//!     -> Result<WorthQueryApplicationCorrectionOutcome, WorthQueryApplicationCorrectionDenial> {
//!     prepared.publish()
//! }
//! fn observe<S>(read: &dyn ApplicationCorrectionPreImage<S>, locus: &DeclaredPreImageLocus<S>) {
//!     let _value = read.field(locus);
//! }
//! ```
