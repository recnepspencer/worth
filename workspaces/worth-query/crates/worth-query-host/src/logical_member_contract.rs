//! Logical observations cannot become candidate mutation targets.
//!
//! The compiling twin resolves the same complete reader/member header and uses
//! the genuine Entity identity already issued for this operation.
//!
//! ```no_run
//! use worth_query_host::facade::{
//!     declaration::{application_schema::ApplicationSchema,
//!         application_operation::ApplicationMutationBinding},
//!     primary_graph::{DecisionReader, WorthQueryInvariantEntityIdentity,
//!         WorthQueryInvariantGraphMember},
//! };
//! fn target<S, B, M>(
//!     reader: &mut DecisionReader<'_, '_, '_, S, B>,
//!     member: &WorthQueryInvariantGraphMember<S, M>,
//!     entity: &WorthQueryInvariantEntityIdentity<S, M>,
//! ) where S: ApplicationSchema, B: ApplicationMutationBinding<S> {
//!     let _ = member.case();
//!     let _ = reader.mutation_target(entity);
//! }
//! ```
//!
//! A projected graph member carries source observation, never entity-write custody.
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{
//!     declaration::{application_schema::ApplicationSchema,
//!         application_operation::ApplicationMutationBinding},
//!     primary_graph::{DecisionReader, WorthQueryInvariantEntityIdentity,
//!         WorthQueryInvariantGraphMember},
//! };
//! fn target<S, B, M>(
//!     reader: &mut DecisionReader<'_, '_, '_, S, B>,
//!     member: &WorthQueryInvariantGraphMember<S, M>,
//!     entity: &WorthQueryInvariantEntityIdentity<S, M>,
//! ) where S: ApplicationSchema, B: ApplicationMutationBinding<S> {
//!     let _ = entity;
//!     let _ = reader.mutation_target(member);
//! }
//! ```
