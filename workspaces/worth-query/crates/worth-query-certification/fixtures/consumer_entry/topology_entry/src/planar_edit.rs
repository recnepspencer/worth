use std::marker::PhantomData;

use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
};

use super::*;

worth_query_operation!(pub EditPlanar for Schema: TopologySchemaBinding, input PlanarMutationInputBinding);
worth_query_operation_reads!(EditPlanar => [Body, BodyKey, PositionX, PositionY, Length, PlanarSuccessor]);
worth_query_operation_writes!(EditPlanar => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(EditPlanar => [Body]);
worth_query_operation_links!(EditPlanar => [PlanarSuccessor]);
worth_query_operation_unlinks!(EditPlanar => [PlanarSuccessor]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarEdit(pub PlanarMutation);

impl From<PlanarMutation> for PlanarEdit {
    fn from(input: PlanarMutation) -> Self {
        Self(input)
    }
}

pub struct PlanarEditBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for PlanarEditBinding<Schema>
{
    type Input = PlanarMutation;
    type InputBinding = PlanarMutationInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = EditPlanar;
    type Decision = ();
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.planar-edit.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.planar-edit-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.planar-edit-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(16, 16, 16, 64, 8192, 4096);

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        super::mutation_identity::key_identity(*key)
    }

    fn input_identity(input: &PlanarMutation) -> [u8; 32] {
        super::mutation_identity::input_identity(input)
    }

    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for PlanarEdit {
    type Binding = PlanarEditBinding<Schema>;

    fn input(&self) -> &PlanarMutation {
        &self.0
    }

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.0.scope_key.clone())
    }
}
