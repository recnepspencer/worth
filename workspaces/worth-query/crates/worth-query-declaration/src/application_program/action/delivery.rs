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
    Mode = super::ApplicationFixedDeliveryMode<Self::Root, Self::Read>,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    type Root: ApplicationOutputGraphShape<Schema> + ApplicationFixedOutputRoot;
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
    read: crate::application_query::ApplicationQueryBindingDescriptor,
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
            read: Delivery::Read::descriptor(),
        }
    }
    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn root_type(&self) -> TypeId {
        self.root
    }
    pub fn read(&self) -> &crate::application_query::ApplicationQueryBindingDescriptor {
        &self.read
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
        let result_limit = self.read.limits().maximum_results().to_string();
        let work_limit = self
            .read
            .limits()
            .maximum_work()
            .map_or_else(|| "host-policy".to_owned(), |limit| limit.to_string());
        super::super::semantic_encoding::framed_parts(&[
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
            self.read.identity(),
            self.read.input_identity().as_str(),
            self.read.query_identity().as_str(),
            self.read.parameter_identity().as_str(),
            self.read.result_identity().as_str(),
            self.read.scope_identity().as_str(),
        ])
    }
}
