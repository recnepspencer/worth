//! Compiler contract for owning native commands through the application audience.
//!
//! The same original owner receives fresh requests; completion transfers evidence only after close.
//!
//! ```
//! use worth_query_host::facade::{
//!     application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition, domain::ApplicationSchema,
//! };
//! fn advance<'r, S, P, D>(
//!     command: WorthQueryCapabilityCommand<'r, S, P, D>,
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//! ) where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
//! {
//!     match command.advance(request) {
//!         WorthQueryCapabilityCommandAdvance::Completed(done) => {
//!             let _native_receipt = done.evidence().receipt();
//!             let _closed_or_same_owner = done.close_with_evidence(request);
//!         }
//!         WorthQueryCapabilityCommandAdvance::Progress(owner) => {
//!             let _same_owner = owner;
//!         }
//!         WorthQueryCapabilityCommandAdvance::Refused { command, denial } => {
//!             let _original_custody_and_cause = (command, denial);
//!         }
//!         WorthQueryCapabilityCommandAdvance::Terminal(owner) => {
//!             let _native_posture = owner.native_outcome();
//!             let _closed_or_same_owner = owner.close(request);
//!         }
//!     }
//! }
//! ```
//!
//! A command cannot be cloned, manually executed, or generically recovered. These
//! refusals use the same otherwise-valid owner and runtime constraints as above.
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition, domain::ApplicationSchema,
//! };
//! fn manual<'r, S, P, D>(command: WorthQueryCapabilityCommand<'r, S, P, D>)
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
//! {
//!     let _ = command.clone();
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition, domain::ApplicationSchema,
//! };
//! fn manual<'r, S, P, D>(command: WorthQueryCapabilityCommand<'r, S, P, D>)
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
//! {
//!     let _ = command.execute();
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition, domain::ApplicationSchema,
//! };
//! fn manual<'r, S, P, D>(command: WorthQueryCapabilityCommand<'r, S, P, D>)
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
//! {
//!     let _ = command.recover();
//! }
//! ```
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::{
//!     application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition, domain::ApplicationSchema,
//! };
//! fn manual<'r, S, P, D>(command: WorthQueryCapabilityCommand<'r, S, P, D>)
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     WorthQueryProgramApplicationRuntime<S, P>: Send + Sync + 'static,
//! {
//!     let _ = command.backend;
//! }
//! ```
//!
//! Original construction is owning; the removed flat ordinary entry is refused.
//!
//! ```
//! use worth_query_host::facade::{
//!     application_contribution::WorthQueryAdvancementPhase,
//!     application_entry::*,
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::{
//!         application_capability::*, application_program::ApplicationProgramDefinition,
//!         application_schema::*,
//!     },
//!     domain::ApplicationSchema,
//!     primary_graph::*,
//! };
//! fn construct<
//!     'command,
//!     Schema: ApplicationSchema,
//!     Program,
//!     PrincipalBinding,
//!     Mapping,
//!     Principal,
//!     PrincipalIdentity,
//!     PrincipalIdentityBinding,
//!     Capability,
//!     Operation,
//!     Input,
//!     PreparationDenial,
//!     Prepare,
//! >(
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
//!     program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
//!     principal_binding: ApplicationPrincipalBindingRef<
//!         Schema,
//!         PrincipalBinding,
//!         Mapping,
//!         Principal,
//!         PrincipalIdentity,
//!         PrincipalIdentityBinding,
//!     >,
//!     capability: ApplicationCapabilityRef<Schema, Capability>,
//!     operation: ApplicationOperationRef<Schema, Operation, Input>,
//!     input: Input,
//!     key: u64,
//!     preconditions: TypedMutationPreconditions<
//!         Schema,
//!         Operation,
//!         <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
//!     >,
//!     mut prepare: Prepare,
//! ) where
//!     Schema: 'command,
//!     PreparationDenial: 'command,
//!     Input: 'command,
//!     Operation: 'command,
//!     Capability: 'command,
//!     PrincipalIdentityBinding: 'command,
//!     PrincipalIdentity: 'command,
//!     Principal: 'command,
//!     Mapping: 'command,
//!     PrincipalBinding: 'command,
//!     Program: 'command,
//!     Prepare: FnMut(
//!             &WorthQueryAdvancementPhase<'_>,
//!             WorthQueryAdmittedApplicationOperation<
//!                 Schema,
//!                 Operation,
//!                 Input,
//!                 <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
//!             >,
//!         ) -> Result<
//!             WorthQueryCapabilityRevocationProgram<
//!                 Schema,
//!                 Operation,
//!                 Input,
//!                 <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
//!             >,
//!             PreparationDenial,
//!         > + 'command,
//!     Program: ApplicationProgramDefinition<Schema>,
//!     PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
//!     PrincipalIdentity: 'static,
//!     Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
//!     Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
//!     Input: ApplicationCapabilityRequest<Schema, Capability>
//!         + ApplicationCapabilityRevocationRequest<Schema, Capability>
//!         + Clone
//!         + Send
//!         + Sync
//!         + 'static,
//! {
//!     let _ = request.retain_capability_revocation_in_program(
//!         program,
//!         principal_binding,
//!         capability,
//!         operation,
//!         input,
//!         key,
//!         preconditions,
//!         prepare,
//!     );
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_contribution::WorthQueryAdvancementPhase,
//!     application_entry::*,
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::{
//!         application_capability::*, application_program::ApplicationProgramDefinition,
//!         application_schema::*,
//!     },
//!     domain::ApplicationSchema,
//!     primary_graph::*,
//! };
//! fn construct<
//!     'command,
//!     Schema: ApplicationSchema,
//!     Program,
//!     PrincipalBinding,
//!     Mapping,
//!     Principal,
//!     PrincipalIdentity,
//!     PrincipalIdentityBinding,
//!     Capability,
//!     Operation,
//!     Input,
//!     PreparationDenial,
//!     Prepare,
//! >(
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
//!     program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
//!     principal_binding: ApplicationPrincipalBindingRef<
//!         Schema,
//!         PrincipalBinding,
//!         Mapping,
//!         Principal,
//!         PrincipalIdentity,
//!         PrincipalIdentityBinding,
//!     >,
//!     capability: ApplicationCapabilityRef<Schema, Capability>,
//!     operation: ApplicationOperationRef<Schema, Operation, Input>,
//!     input: Input,
//!     key: u64,
//!     preconditions: TypedMutationPreconditions<
//!         Schema,
//!         Operation,
//!         <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
//!     >,
//!     mut prepare: Prepare,
//! ) where
//!     Schema: 'command,
//!     PreparationDenial: 'command,
//!     Input: 'command,
//!     Operation: 'command,
//!     Capability: 'command,
//!     PrincipalIdentityBinding: 'command,
//!     PrincipalIdentity: 'command,
//!     Principal: 'command,
//!     Mapping: 'command,
//!     PrincipalBinding: 'command,
//!     Program: 'command,
//!     Prepare: FnMut(
//!             &WorthQueryAdvancementPhase<'_>,
//!             WorthQueryAdmittedApplicationOperation<
//!                 Schema,
//!                 Operation,
//!                 Input,
//!                 <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
//!             >,
//!         ) -> Result<
//!             WorthQueryCapabilityRevocationProgram<
//!                 Schema,
//!                 Operation,
//!                 Input,
//!                 <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
//!             >,
//!             PreparationDenial,
//!         > + 'command,
//!     Program: ApplicationProgramDefinition<Schema>,
//!     PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
//!     PrincipalIdentity: 'static,
//!     Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
//!     Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
//!     Input: ApplicationCapabilityRequest<Schema, Capability>
//!         + ApplicationCapabilityRevocationRequest<Schema, Capability>
//!         + Clone
//!         + Send
//!         + Sync
//!         + 'static,
//! {
//!     let _ = request.execute_capability_revocation_in_program(
//!         program,
//!         principal_binding,
//!         capability,
//!         operation,
//!         input,
//!         &key,
//!         preconditions,
//!         prepare,
//!     );
//! }
//! ```
