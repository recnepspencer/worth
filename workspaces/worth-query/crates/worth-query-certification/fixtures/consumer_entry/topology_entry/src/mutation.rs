use super::*;
use std::marker::PhantomData;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarMutation {
    pub scope_key: String,
    pub operation: PlanarOperation,
    pub validator_work: usize,
}
worth_query_structured_value_binding!(pub PlanarMutationInputBinding for PlanarMutation { identity: "worth.query.certification.planar-mutation-input.v1" });
worth_query_structured_value_binding!(pub PlanarMutationResultBinding for PlanarAdjustmentResult { identity: "worth.query.certification.planar-mutation-result.v1" });
worth_query_structured_value_binding!(pub PlanarMutationDenialBinding for PlanarMutationDenial { identity: "worth.query.certification.planar-mutation-denial.v1" });
worth_query_operation!(pub MutatePlanar for Schema: TopologySchemaBinding, input PlanarMutationInputBinding);
worth_query_operation_reads!(MutatePlanar => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_writes!(MutatePlanar => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(MutatePlanar => [Body]);
worth_query_operation_links!(MutatePlanar => [PlanarSuccessor]);

pub struct PlanarMutationBinding<Schema>(PhantomData<fn() -> Schema>);
pub type PlanarMutationScope<Schema> = ApplicationMutationFieldScope<
    Schema,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for PlanarMutationBinding<Schema>
{
    type Input = PlanarMutation;
    type InputBinding = PlanarMutationInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = MutatePlanar;
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
    const IDENTITY: &'static str = "worth.query.certification.planar-mutation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.planar-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.certification.planar-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(16, 16, 64, 8192, 4096);
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
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for PlanarMutation {
    type Binding = PlanarMutationBinding<Schema>;
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
pub const fn requirements(
    creates: usize,
    links: usize,
    writes: usize,
    bytes: usize,
    work: usize,
) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(creates, 0, links, 0, writes, 0),
        ApplicationCandidateResourceCeiling::bounded(bytes, work),
    )
}

pub struct PlanarOutputs;
impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema> for PlanarOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            Schema,
            Body,
        >(
            "anchor", ApplicationMutationOutputPosture::Preserve
        )];
}
