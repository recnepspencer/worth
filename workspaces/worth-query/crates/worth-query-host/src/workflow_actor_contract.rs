//! Explicit workflow declarations and positive recorded-role requirements.
//!
//! A typed declaration pairs one binding with the original native occurrence.
//! This function only authors meaning; installation and native admission still
//! check the installed contract and the actual original command.
//!
//! ```
//! use worth_query_host::facade::declaration::{
//!     application_capability::*, application_operation::*, application_program::*,
//! };
//! fn workflow_actor_contract_approval<Spec, Binding>() -> (
//!     ApplicationWorkflowActorPolicy, ApplicationCapabilityRecordedActorRule,
//! ) where Spec: ApplicationWorkflowSpec,
//!     Binding: ApplicationCapabilityMutationBinding<Spec::Schema>,
//! {
//!     let source = ApplicationWorkflowRecordedActorSource::BoundProposal;
//!     (ApplicationWorkflowActorPolicy::for_approval::<Spec, Binding>(source),
//!      ApplicationCapabilityRecordedActorRule::required(
//!          ApplicationCapabilityRecordedActorRequirement::new(source,
//!              ApplicationWorkflowRecordedRole::Proposer,
//!              ApplicationRecordedActorComparison::DifferentPrincipal)))
//! }
//! ```
//!
//! Every spec supplies both policies. The matching negative below has the same
//! schema and lifetime bounds, so missing policy methods are the decisive error.
//!
//! ```
//! use worth_query_host::facade::declaration::{application_schema::ApplicationSchema,
//!     application_program::*};
//! struct Workflow<S>(std::marker::PhantomData<S>);
//! impl<S: ApplicationSchema + 'static> ApplicationWorkflowSpec for Workflow<S> {
//!     type Schema = S;
//!     const IDENTITY: ApplicationWorkflowSpecIdentity =
//!         ApplicationWorkflowSpecIdentity::new("workflow.actor-contract.v1");
//!     fn observation_policy() -> ApplicationWorkflowObservationPolicy {
//!         ApplicationWorkflowObservationPolicy::disabled()
//!     }
//!     fn actor_policy() -> ApplicationWorkflowActorPolicy {
//!         ApplicationWorkflowActorPolicy::not_applicable()
//!     }
//! }
//! ```
//!
//! ```compile_fail,E0046
//! use worth_query_host::facade::declaration::{application_schema::ApplicationSchema,
//!     application_program::*};
//! struct Workflow<S>(std::marker::PhantomData<S>);
//! impl<S: ApplicationSchema + 'static> ApplicationWorkflowSpec for Workflow<S> {
//!     type Schema = S;
//!     const IDENTITY: ApplicationWorkflowSpecIdentity =
//!         ApplicationWorkflowSpecIdentity::new("workflow.actor-contract.v1");
//! }
//! ```
//!
//! The previous empty constructor is absent. Explicit occurrence-only meaning
//! uses the same otherwise-valid type without optional/default interpretation.
//!
//! ```
//! use worth_query_host::facade::declaration::application_operation::ApplicationWorkflowExecutionRequirements;
//! let occurrence = ApplicationWorkflowExecutionRequirements::occurrence_only();
//! assert!(occurrence.is_applicable());
//! let ordinary = ApplicationWorkflowExecutionRequirements::not_applicable();
//! assert!(!ordinary.is_applicable());
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::declaration::application_operation::ApplicationWorkflowExecutionRequirements;
//! let occurrence = ApplicationWorkflowExecutionRequirements::new();
//! ```
//!
//! These two expansions isolate the same requirement arm used by a guarded
//! mutation declaration. No expected result type can hide a restored omitted
//! branch behind an unrelated type error.
//!
//! ```
//! use worth_query_host::facade::{worth_query_mutation_binding,
//!     declaration::application_operation::ApplicationWorkflowExecutionRequirements};
//! let _ = worth_query_mutation_binding!(@workflow_requirements [required]
//!     [ApplicationWorkflowExecutionRequirements::occurrence_only()]);
//! ```
//!
//! ```compile_fail
//! use worth_query_host::facade::worth_query_mutation_binding;
//! let _ = worth_query_mutation_binding!(@workflow_requirements [required] []);
//! ```
