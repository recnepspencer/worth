//! Installed action delivery is canonical meaning, never request authority.
use super::super::{
    ApplicationConnectionShape, ApplicationFixedOutputRoot, ApplicationOutputGraphShape,
};
use crate::{
    application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};
use std::any::TypeId;

/// The installed declaration binds this concrete implementation in the running process.
/// `IDENTITY` supplies stable canonical meaning; it does not admit another selector type.
pub trait ApplicationFixedActionDelivery<Schema, Binding>:
    super::ApplicationActionDelivery<
    Schema,
    Binding,
    Mode = super::ApplicationFixedDeliveryMode<Self::Root, Self::Projection>,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Root: ApplicationOutputGraphShape<Schema> + ApplicationFixedOutputRoot;
    type Projection: super::ApplicationFixedProjection<Schema>;
}

/// A projected fixed delivery binds one concrete query and its original-input selector.
pub trait ApplicationFixedQueryDelivery<Schema, Binding>:
    ApplicationFixedActionDelivery<
    Schema,
    Binding,
    Projection = super::ApplicationQueryProjection<Self::Read>,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Read: ApplicationQueryBinding<Schema>;
    fn read_input(input: &Binding::Input)
        -> <Self::Read as ApplicationQueryBinding<Schema>>::Input;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFixedActionDeliveryDeclaration {
    identity: &'static str,
    // Runtime-only binding, like the root stamp; never part of canonical encoding.
    implementation: TypeId,
    root: TypeId,
    root_connection: super::super::ApplicationConnectionDeclaration,
    root_connections: Box<[super::super::ApplicationConnectionDeclaration]>,
    read: Option<crate::application_query::ApplicationQueryBindingDescriptor>,
    batch_items: Option<usize>,
}
impl ApplicationFixedActionDeliveryDeclaration {
    pub fn fixed<Schema, Binding, Delivery>() -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Delivery: ApplicationFixedActionDelivery<Schema, Binding>,
    {
        let root_connections =
            <Delivery::Root as super::super::ApplicationProgramOutputShape<Schema>>::connections();
        Self {
            root_connections: root_connections.into_boxed_slice(),
            identity: <Delivery as super::ApplicationActionDelivery<Schema, Binding>>::IDENTITY,
            implementation: TypeId::of::<Delivery>(),
            root: TypeId::of::<Delivery::Root>(),
            root_connection:
                <Delivery::Root as ApplicationOutputGraphShape<Schema>>::RootConnection::declaration(
                ),
            read: <Delivery::Projection as super::ApplicationFixedProjection<Schema>>::descriptor(),
            batch_items:
                <Delivery::Projection as super::ApplicationFixedProjection<Schema>>::batch_items(),
        }
    }
    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn root_type(&self) -> TypeId {
        self.root
    }
    pub fn read(&self) -> Option<&crate::application_query::ApplicationQueryBindingDescriptor> {
        self.read.as_ref()
    }
    pub const fn batch_items(&self) -> Option<usize> {
        self.batch_items
    }
    pub fn root_connection(&self) -> &super::super::ApplicationConnectionDeclaration {
        &self.root_connection
    }
    pub fn canonical_meaning(&self) -> String {
        let mut graph = self
            .root_connections
            .iter()
            .map(|connection| {
                super::super::semantic_encoding::framed_parts(&[
                    connection.identity(),
                    connection.source_instance(),
                    connection.source_feature(),
                    connection.source_port(),
                    connection.target_instance(),
                    connection.target_feature(),
                    connection.target_port(),
                ])
            })
            .collect::<Vec<_>>();
        graph.sort_unstable();
        let graph = super::super::semantic_encoding::framed_parts(
            &graph.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        let Some(read) = &self.read else {
            return super::super::semantic_encoding::framed_parts(&[
                self.identity,
                &graph,
                "fixed-no-projection",
                self.root_connection.identity(),
                self.root_connection.source_instance(),
                self.root_connection.source_feature(),
                self.root_connection.source_port(),
                self.root_connection.target_instance(),
                self.root_connection.target_feature(),
                self.root_connection.target_port(),
            ]);
        };
        let result_limit = read.limits().maximum_results().to_string();
        let work_limit = read
            .limits()
            .maximum_work()
            .map_or_else(|| "host-policy".to_owned(), |limit| limit.to_string());
        let query_meaning = super::super::semantic_encoding::framed_parts(&[
            self.identity,
            &graph,
            &result_limit,
            &work_limit,
            self.root_connection.identity(),
            self.root_connection.source_instance(),
            self.root_connection.source_feature(),
            self.root_connection.source_port(),
            self.root_connection.target_instance(),
            self.root_connection.target_feature(),
            self.root_connection.target_port(),
            read.identity(),
            read.input_identity().as_str(),
            read.query_identity().as_str(),
            read.parameter_identity().as_str(),
            read.result_identity().as_str(),
            read.scope_identity().as_str(),
        ]);
        match self.batch_items {
            Some(items) => super::super::semantic_encoding::framed_parts(&[
                "fixed-query-batch",
                &items.to_string(),
                &query_meaning,
            ]),
            None => query_meaning,
        }
    }
}
