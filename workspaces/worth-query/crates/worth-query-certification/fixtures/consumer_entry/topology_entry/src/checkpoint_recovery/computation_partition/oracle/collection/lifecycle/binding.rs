//! Exact lifecycle roles and binding-indexed authored scope selection.
use super::*;
use worth_query_decl::facade::application_schema::*;

macro_rules! binding {
    ($binding:ident, $outputs:ident, $role:ident, $support:ident, $extra:ident, $action:ty, $postures:ident, $extra_postures:ident, $id:literal) => {
        pub(in crate::checkpoint_recovery::computation_partition::oracle::collection) struct $outputs;
        pub(in crate::checkpoint_recovery::computation_partition::oracle::collection) struct $role<Schema>(PhantomData<fn() -> Schema>);
        impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole for $role<Schema> {
            type Schema = Schema;
            type Contract = $outputs;
            type Entity = Body;
            type Action = $action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = "total";
        }
        pub(in crate::checkpoint_recovery::computation_partition::oracle::collection) struct $support<Schema>(PhantomData<fn() -> Schema>);
        impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRoleFamily for $support<Schema> {
            type Schema = Schema;
            type Contract = $outputs;
            type Entity = Body;
            const PREFIX: &'static str = "support.";
            const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::$postures;
            const MINIMUM: usize = 2;
        }
        impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema> for $outputs {
            const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                &[<$role<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
            const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
                &[<$support<Schema> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
                  <$extra<Schema> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
        }
        pub(super) struct $extra<Schema>(PhantomData<fn() -> Schema>);
        impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRoleFamily for $extra<Schema> {
            type Schema = Schema;
            type Contract = $outputs;
            type Entity = members::GeneratedSupport;
            const PREFIX: &'static str = "optional.";
            const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::$extra_postures;
            const MINIMUM: usize = 0;
        }
        pub(in crate::checkpoint_recovery::computation_partition::oracle::collection) struct $binding<Schema>(PhantomData<fn() -> Schema>);
        impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
            for $binding<Schema>
        {
            type Input = RegionOutputInput;
            type InputBinding = RegionOutputInputBinding;
            type Operation = TotalRegionOutput;
            type Decision = (u64, bool, u64);
            type Output = $outputs;
            type Result = worth_query_consumer_values::PlanarAdjustmentResult;
            type ResultBinding = PlanarMutationResultBinding;
            type IdempotencyKey = u64;
            type Denial = worth_query_consumer_values::PlanarMutationDenial;
            type DenialBinding = PlanarMutationDenialBinding;
            type ScopeBinding = PlanarMutationScope<Schema>;
            type PrincipalBinding = ConsumerPrincipalBinding;
            type Mapping = ExternalPrincipalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
            const IDENTITY: &'static str = concat!("collection-lifecycle-", $id);
            const HANDLER_IDENTITY: &'static str = concat!("collection-lifecycle-handler-", $id);
            const IDEMPOTENCY_IDENTITY: &'static str =
                concat!("collection-lifecycle-command-", $id);

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
                input: &RegionOutputInput,
            ) -> Result<
                Option<
                    worth_query_decl::facade::application_query::ApplicationQueryParameterSet<
                        PlanarQuery,
                    >,
                >,
                ApplicationValueEncodeDenial,
            > {
                use worth_query_decl::facade::application_query::ApplicationQueryIntent;
                Ok(Some(
                    <PlanarRead as ApplicationQueryIntent<Schema>>::parameters(&PlanarRead {
                        body_key: input.scope_key.clone(),
                    }),
                ))
            }
        }
    };
}
binding!(
    InitialBinding,
    InitialOutputs,
    CreatedTotal,
    CreatedSupport,
    CreatedExtra,
    WorthQueryCreateOutput,
    CREATE,
    CREATE,
    "initial"
);
binding!(
    PreserveBinding,
    PreserveOutputs,
    PreservedTotal,
    PreservedSupport,
    PreservedExtra,
    WorthQueryPreserveOutput,
    PRESERVE,
    ALL,
    "preserve"
);

// compiled-example: begin
impl ApplicationMutationInputScope<CheckpointSchema, InitialBinding<CheckpointSchema>>
    for RegionOutputInput
{
    fn scope_for_binding(&self) -> PlanarMutationScope<CheckpointSchema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
impl ApplicationMutationInputScope<CheckpointSchema, PreserveBinding<CheckpointSchema>>
    for RegionOutputInput
{
    fn scope_for_binding(&self) -> PlanarMutationScope<CheckpointSchema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
// compiled-example: end
