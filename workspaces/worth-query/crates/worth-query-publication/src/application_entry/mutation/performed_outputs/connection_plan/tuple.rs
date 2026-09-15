use super::*;

trait ProgramConnectionPlanMember<Schema, Program, Inventory>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    fn append<'application>(
        connections: &mut Vec<
            Box<
                dyn ErasedProgramConnection<'application, Schema, Program, Inventory>
                    + 'application,
            >,
        >,
    ) where
        Schema: 'application,
        Program: 'application,
        Inventory: 'application;
}

macro_rules! impl_empty_member {
    ($wrapper:ident) => {
        impl<Schema, Program, Inventory, Connection>
            ProgramConnectionPlanMember<Schema, Program, Inventory> for $wrapper<Connection>
        where
            Schema: ApplicationSchema,
            Program: ApplicationProgramDefinition<Schema>,
            Inventory: ApplicationProgramInventoryIdentity,
        {
            fn append<'application>(
                _: &mut Vec<
                    Box<
                        dyn ErasedProgramConnection<'application, Schema, Program, Inventory>
                            + 'application,
                    >,
                >,
            ) where
                Schema: 'application,
                Program: 'application,
                Inventory: 'application,
            {
            }
        }
    };
}

impl_empty_member!(ApplicationProgramRequiredConnection);
impl_empty_member!(ApplicationProgramUnavailableConnection);

impl<Schema, Program, Inventory, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    ProgramConnectionPlanMember<Schema, Program, Inventory>
    for ApplicationProgramDependentConnection<
        Connection<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>,
    >
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<Schema, TargetFeature, Value = SourcePort::Value>,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>
        + WorthQueryApplicationDependentOutputConnection<Schema>,
    TypedDependentConnection<Schema, Program, Inventory, SourceFeature, TargetFeature, Binding>:
        for<'application> ErasedProgramConnection<'application, Schema, Program, Inventory>,
{
    fn append<'application>(
        connections: &mut Vec<
            Box<
                dyn ErasedProgramConnection<'application, Schema, Program, Inventory>
                    + 'application,
            >,
        >,
    ) where
        Schema: 'application,
        Program: 'application,
        Inventory: 'application,
    {
        connections.push(Box::new(TypedDependentConnection::<
            Schema,
            Program,
            Inventory,
            SourceFeature,
            TargetFeature,
            Binding,
        >(PhantomData)));
    }
}

impl<Schema, Program, Inventory> WorthQueryProgramConnectionPlan<Schema, Program, Inventory> for ()
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
{
    fn connections<'application>(
    ) -> super::super::WorthQueryProgramConnectionFactories<'application, Schema, Program, Inventory>
    where
        Schema: 'application,
        Program: 'application,
        Inventory: 'application,
    {
        super::super::WorthQueryProgramConnectionFactories::new(Vec::new())
    }
}

macro_rules! impl_connection_plan {
    ($($member:ident),+) => {
        impl<Schema, Program, Inventory, $($member),+>
            WorthQueryProgramConnectionPlan<Schema, Program, Inventory> for ($($member,)+)
        where
            Schema: ApplicationSchema,
            Program: ApplicationProgramDefinition<Schema>,
            Inventory: ApplicationProgramInventoryIdentity,
            $($member: ProgramConnectionPlanMember<Schema, Program, Inventory>,)+
        {
            fn connections<'application>() -> super::super::WorthQueryProgramConnectionFactories<'application, Schema, Program, Inventory>
            where Schema: 'application, Program: 'application, Inventory: 'application {
                let mut connections = Vec::new();
                $($member::append(&mut connections);)+
                super::super::WorthQueryProgramConnectionFactories::new(connections)
            }
        }
    };
}

impl_connection_plan!(A);
impl_connection_plan!(A, B);
impl_connection_plan!(A, B, C);
impl_connection_plan!(A, B, C, D);
impl_connection_plan!(A, B, C, D, E);
impl_connection_plan!(A, B, C, D, E, F);
impl_connection_plan!(A, B, C, D, E, F, G);
impl_connection_plan!(A, B, C, D, E, F, G, H);
impl_connection_plan!(A, B, C, D, E, F, G, H, I);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J, K);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_connection_plan!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
