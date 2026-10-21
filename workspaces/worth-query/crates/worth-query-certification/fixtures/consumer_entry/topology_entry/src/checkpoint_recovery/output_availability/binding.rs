//! One operation and canonical input, selected by two exact native lifecycle bindings.
use super::*;
use members::{
    DemandKey, MemberKind, MemberNumber, PopulationMember, Quantity, QuantityKey, QuantitySnapshot,
};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct Input {
    pub scope_key: String,
}
worth_query_structured_value_binding!(pub(super) InputBinding for Input {identity:"availability-publication-input.v1"});
worth_query_operation!(pub(super) PublishQuantity for Schema: TopologySchemaBinding, input InputBinding);
worth_query_operation_reads!(PublishQuantity => [Body,BodyKey,Length,PopulationMember,MemberNumber,DemandKey,MemberKind,Quantity,QuantityKey,QuantitySnapshot]);
worth_query_operation_writes!(PublishQuantity => [QuantityKey,QuantitySnapshot]);
worth_query_operation_creates!(PublishQuantity => [Quantity]);
macro_rules! binding {
    ($binding:ident,$outputs:ident,$role:ident,$action:ty,$id:literal) => {
        pub(super) struct $binding;
        pub(super) struct $outputs;
        pub(super) struct $role;
        impl WorthQueryApplicationOutputRole for $role {
            type Schema = Schema;
            type Contract = $outputs;
            type Entity = Quantity;
            type Action = $action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = "quantity";
        }
        impl ApplicationMutationOutputContract<Schema> for $outputs {
            const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                &[<$role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
        }
        impl ApplicationMutationBinding<Schema> for $binding {
            type Input = Input;
            type InputBinding = InputBinding;
            type Operation = PublishQuantity;
            type Decision = Snapshot;
            type Output = $outputs;
            type Result = PlanarAdjustmentResult;
            type ResultBinding = PlanarMutationResultBinding;
            type IdempotencyKey = u64;
            type Denial = PlanarMutationDenial;
            type DenialBinding = PlanarMutationDenialBinding;
            type ScopeBinding = PlanarMutationScope<Schema>;
            type PrincipalBinding = ConsumerPrincipalBinding;
            type Mapping = ExternalPrincipalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
            const IDENTITY: &'static str = concat!("availability-", $id);
            const HANDLER_IDENTITY: &'static str = concat!("availability-handler-", $id);
            const IDEMPOTENCY_IDENTITY: &'static str = concat!("availability-command-", $id);

            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
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
            fn expected_source_parameters(
                input: &Input,
            ) -> Result<
                Option<ApplicationQueryParameterSet<PlanarQuery>>,
                ApplicationValueEncodeDenial,
            > {
                Ok(Some(
                    <PlanarRead as ApplicationQueryIntent<Schema>>::parameters(&PlanarRead {
                        body_key: input.scope_key.clone(),
                    }),
                ))
            }
        }
        impl ApplicationMutationInputScope<Schema, $binding> for Input {
            fn scope_for_binding(&self) -> PlanarMutationScope<Schema> {
                PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
            }
        }
    };
}
binding!(
    Initial,
    InitialOutputs,
    Created,
    WorthQueryCreateOutput,
    "initial-v1"
);
binding!(
    Preserve,
    PreserveOutputs,
    Preserved,
    WorthQueryPreserveOutput,
    "preserve-v1"
);
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = PublishQuantity::reference::<Schema>();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        // The reader funds the complete governed source Query preparation,
        // not just native field visits (native7 required 81,768 work already).
        .operation_projection_work_budget(op, 1_000_000)
        .operation_read_entity(op, Body::reference())
        .operation_read_entity(op, Quantity::reference())
        .operation_read_field(op, BodyKey::reference())
        .operation_read_field(op, Length::reference())
        .operation_read_field(op, MemberNumber::reference())
        .operation_read_field(op, DemandKey::reference())
        .operation_read_field(op, MemberKind::reference())
        .operation_read_relation(op, PopulationMember::reference())
        .operation_read_field(op, QuantityKey::reference())
        .operation_read_field(op, QuantitySnapshot::reference())
        .operation_create(op, Quantity::reference())
        .operation_write(op, QuantityKey::reference())
        .operation_write(op, QuantitySnapshot::reference())
        .application_mutation_binding::<Initial>()
        .application_mutation_binding::<Preserve>()
}
