use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationConnectionRef, ApplicationFeature,
    ApplicationInputPort, ApplicationOccurrenceConnectionBinding, ApplicationOutputPort,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationProgramConnectionRole {
    RequiredSource,
    Dependent,
    Unavailable,
}

pub struct ApplicationProgramRequiredConnection<Connection>(PhantomData<fn() -> Connection>);
pub struct ApplicationProgramDependentConnection<Connection>(PhantomData<fn() -> Connection>);
pub struct ApplicationProgramUnavailableConnection<Connection>(PhantomData<fn() -> Connection>);

pub trait ApplicationProgramConnectionNode<Schema>
where
    Schema: ApplicationSchema,
{
    fn declaration() -> ApplicationConnectionDeclaration;
}

macro_rules! impl_connection_node {
    ($wrapper:ident, $role:expr) => {
        impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
            ApplicationProgramConnectionNode<Schema>
            for $wrapper<
                ApplicationConnectionRef<
                    Schema,
                    SourceFeature,
                    SourcePort,
                    TargetFeature,
                    TargetPort,
                    Binding,
                >,
            >
        where
            Schema: ApplicationSchema,
            SourceFeature: ApplicationFeature<Schema>,
            TargetFeature: ApplicationFeature<Schema>,
            SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
            TargetPort: ApplicationInputPort<Schema, TargetFeature, Value = SourcePort::Value>,
            Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
        {
            fn declaration() -> ApplicationConnectionDeclaration {
                ApplicationConnectionRef::<
                    Schema,
                    SourceFeature,
                    SourcePort,
                    TargetFeature,
                    TargetPort,
                    Binding,
                >::declaration($role, std::any::TypeId::of::<Self>())
            }
        }
    };
}

impl_connection_node!(
    ApplicationProgramRequiredConnection,
    ApplicationProgramConnectionRole::RequiredSource
);
impl_connection_node!(
    ApplicationProgramDependentConnection,
    ApplicationProgramConnectionRole::Dependent
);
impl_connection_node!(
    ApplicationProgramUnavailableConnection,
    ApplicationProgramConnectionRole::Unavailable
);

pub trait ApplicationProgramConnectionSet<Schema>
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationConnectionDeclaration>;
}

impl<Schema> ApplicationProgramConnectionSet<Schema> for ()
where
    Schema: ApplicationSchema,
{
    fn declarations() -> Vec<ApplicationConnectionDeclaration> {
        Vec::new()
    }
}

macro_rules! impl_connection_sets {
    ($($connection:ident),+) => {
        impl<Schema, $($connection),+> ApplicationProgramConnectionSet<Schema>
            for ($($connection,)+)
        where
            Schema: ApplicationSchema,
            $($connection: ApplicationProgramConnectionNode<Schema>,)+
        {
            fn declarations() -> Vec<ApplicationConnectionDeclaration> {
                vec![$($connection::declaration()),+]
            }
        }
    };
}

impl_connection_sets!(A);
impl_connection_sets!(A, B);
impl_connection_sets!(A, B, C);
impl_connection_sets!(A, B, C, D);
impl_connection_sets!(A, B, C, D, E);
impl_connection_sets!(A, B, C, D, E, F);
impl_connection_sets!(A, B, C, D, E, F, G);
impl_connection_sets!(A, B, C, D, E, F, G, H);
impl_connection_sets!(A, B, C, D, E, F, G, H, I);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J, K);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_connection_sets!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
