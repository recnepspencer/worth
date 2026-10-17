//! Checked collection installation through the ordinary declaration/host facades.
//!
//! ```
//! use worth_query_host::facade::{application_contribution::{
//!     WorthQueryApplicationContributionSetup, WorthQueryInstalledPartitionedComputation,
//!     WorthQueryPartitionedComputationOwner,
//! }, primary_graph::WorthQueryPrimaryGraphInstallationDenial};
//! use worth_query_decl::facade::{application_schema::ApplicationSchema,
//!     application_program::{ApplicationFeature, ApplicationDerivedCollection,
//!         ApplicationCollectionCompleteness}};
//! fn install<S, F, C, O>(setup: &mut WorthQueryApplicationContributionSetup<'_, S>, owner: O)
//!     -> Result<WorthQueryInstalledPartitionedComputation<S, F, C::Computation, O>, WorthQueryPrimaryGraphInstallationDenial>
//! where S: ApplicationSchema, F: ApplicationFeature<S>, C: ApplicationDerivedCollection<S, F>,
//!       O: WorthQueryPartitionedComputationOwner<S, F, C::Computation,
//!           Output = <C::Completeness as ApplicationCollectionCompleteness>::Value,
//!           Stopped = <C::Completeness as ApplicationCollectionCompleteness>::Stopped>,
//! {
//!     setup.derived_collection::<F, C, O>(owner)
//! }
//! ```
//!
//! Output/completeness value mismatch fails at the actual installation method.
//!
//! ```compile_fail,E0271
//! use worth_query_host::facade::application_contribution::{
//!     WorthQueryApplicationContributionSetup, WorthQueryPartitionedComputationOwner,
//! };
//! use worth_query_decl::facade::{application_schema::ApplicationSchema,
//!     application_program::{ApplicationFeature, ApplicationDerivedCollection,
//!         ApplicationCollectionCompleteness}};
//! fn mismatched<S, F, C, O>(setup: &mut WorthQueryApplicationContributionSetup<'_, S>, owner: O)
//! where S: ApplicationSchema, F: ApplicationFeature<S>, C: ApplicationDerivedCollection<S, F>,
//!       C::Completeness: ApplicationCollectionCompleteness<Value = String, Stopped = u32>,
//!       O: WorthQueryPartitionedComputationOwner<S, F, C::Computation, Output = u64, Stopped = u32>,
//! {
//!     setup.derived_collection::<F, C, O>(owner);
//! }
//! ```
//!
//! The old identity-only implementation cannot omit the required associations.
//!
//! ```compile_fail,E0046
//! use worth_query_decl::facade::{application_schema::ApplicationSchema,
//!     application_program::{ApplicationFeature, ApplicationDerivedCollection,
//!         ApplicationCollectionIncompletePosture}};
//! struct Missing<C>(std::marker::PhantomData<C>);
//! impl<S, F, C> ApplicationDerivedCollection<S, F> for Missing<C>
//! where S: ApplicationSchema, F: ApplicationFeature<S>, C: ApplicationDerivedCollection<S, F> {
//!     type Contributor = C::Contributor;
//!     type Grouping = C::Grouping;
//!     type Measures = C::Measures;
//!     type Lineage = C::Lineage;
//!     const IDENTITY: &'static str = C::IDENTITY;
//!     const APPLICABILITY: &'static str = C::APPLICABILITY;
//!     const INCOMPLETE: ApplicationCollectionIncompletePosture = C::INCOMPLETE;
//!     const INCREMENTAL_UPDATE: &'static str = C::INCREMENTAL_UPDATE;
//! }
//! ```
