//! Independent ordinary native mutation moves only collection membership.
use super::{
    draft::{DraftBinding, DraftInput, DraftInputBinding},
    schema::*,
};
use worth_query_decl::facade::application_operation::ApplicationCandidatePolicy;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
    },
};
pub struct UnlinkBinding<const MODE: u64>;
type Scope<const MODE: u64> =
    <DraftBinding<MODE, RequireDraftNote> as ApplicationMutationBinding<
        DocumentRetentionSchema<MODE>,
    >>::ScopeBinding;
pub struct Targets<const MODE: u64> {
    document: WorthQueryInvariantMutationTarget<DocumentRetentionSchema<MODE>, Document>,
    member: WorthQueryInvariantMutationTarget<DocumentRetentionSchema<MODE>, Audit>,
}

impl<const MODE: u64> ApplicationMutationBinding<DocumentRetentionSchema<MODE>>
    for UnlinkBinding<MODE>
{
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = DraftInput;
    type InputBinding = DraftInputBinding;
    type Result = ();
    type ResultBinding = super::binding::BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = UnlinkMember;
    type Decision = Targets<MODE>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = super::binding::BodyDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope<MODE>;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.body-court.unlink-member.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.body-court.unlink-member-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.body-court.unlink-member-command.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    const CANDIDATES: ApplicationCandidatePolicy =
        ApplicationCandidatePolicy::finite(ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 1, 0, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(4096),
        ));
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema<MODE>,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema<MODE>,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

pub struct UnlinkHandler;
impl<const MODE: u64> OperationHandler<DocumentRetentionSchema<MODE>, UnlinkBinding<MODE>>
    for UnlinkHandler
{
    fn decide(
        &self,
        input: &DraftInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema<MODE>, UnlinkBinding<MODE>>,
    ) -> HandlerResult<Targets<MODE>, WorthQueryOperationBodyDenial> {
        let document = match reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
        {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let members = match reader.relations_from(DocumentMembers::reference(), &document) {
            Ok(value) => value,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        if members.len() != 1 {
            return HandlerResult::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch {
                member: "single fixture member".to_owned(),
            });
        }
        let document = match reader.mutation_target(&document) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let member = match reader.mutation_target(members[0].to()) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        HandlerResult::Completed(Targets { document, member })
    }
    fn candidate_requirements(
        &self,
        _: &DraftInput,
        _: &Targets<MODE>,
    ) -> ApplicationCandidateRequirements {
        UnlinkBinding::<MODE>::CANDIDATES
            .finite_requirements()
            .expect("fixed-shape handler declares finite candidate policy")
    }
    fn build_candidate(
        &self,
        _: &DraftInput,
        targets: Targets<MODE>,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema<MODE>, UnlinkBinding<MODE>>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let document = match writer.projected_entity(&targets.document) {
            Ok(value) => value,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let member = match writer.projected_entity(&targets.member) {
            Ok(value) => value,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        match writer.unlink(DocumentMembers::reference(), &document, &member) {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

#[derive(Clone)]
pub struct UnlinkIntent<const MODE: u64>(pub DraftInput);
impl<const MODE: u64> ApplicationMutationIntent<DocumentRetentionSchema<MODE>>
    for UnlinkIntent<MODE>
{
    type Binding = UnlinkBinding<MODE>;
    fn input(&self) -> &DraftInput {
        &self.0
    }
    fn scope_binding(&self) -> Scope<MODE> {
        Scope::<MODE>::new(DocumentIdentityField::reference(), self.0.identity.clone())
    }
}
