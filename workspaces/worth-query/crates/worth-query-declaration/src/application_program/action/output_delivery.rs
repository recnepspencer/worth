//! Canonical output delivery payload shared by genuine fixed and discovered modes.
use super::super::{ApplicationConnectionShape, ApplicationOutputGraphShape};
use super::ApplicationFixedActionDelivery;
use crate::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};
use std::any::TypeId;
/// Installed graph and projection meaning; concrete implementation stamps are runtime-only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationOutputActionDeliveryDeclaration {
    discovered: bool,
    identity: &'static str,
    // Runtime-only binding, like the root stamp; never part of canonical encoding.
    implementation: TypeId,
    root: TypeId,
    root_connection: super::super::ApplicationConnectionDeclaration,
    root_connections: Box<[super::super::ApplicationConnectionDeclaration]>,
    read: Option<crate::application_query::ApplicationQueryBindingDescriptor>,
    batch_items: Option<usize>,
}
impl ApplicationOutputActionDeliveryDeclaration {
    pub fn fixed<Schema, Binding, Delivery>() -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Delivery: ApplicationFixedActionDelivery<Schema, Binding>,
    {
        Self::for_root::<Schema, Binding, Delivery, Delivery::Root, Delivery::Projection>(false)
    }
    pub fn discovered<Schema, Binding, Delivery>() -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Delivery: super::ApplicationDiscoveredActionDelivery<Schema, Binding>,
    {
        Self::for_root::<Schema, Binding, Delivery, Delivery::Root, Delivery::Projection>(true)
    }
    fn for_root<S, B, D, R, Q>(discovered: bool) -> Self
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
        D: super::ApplicationActionDelivery<S, B>,
        R: ApplicationOutputGraphShape<S>,
        Q: super::ApplicationOutputProjection<S>,
    {
        Self {
            discovered,
            root_connections: <R as super::super::ApplicationProgramOutputShape<S>>::connections()
                .into_boxed_slice(),
            identity: D::IDENTITY,
            implementation: TypeId::of::<D>(),
            root: TypeId::of::<R>(),
            root_connection: R::RootConnection::declaration(),
            read: Q::descriptor(),
            batch_items: Q::batch_items(),
        }
    }
    pub const fn is_discovered(&self) -> bool {
        self.discovered
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
                if self.discovered {
                    "discovered-no-projection"
                } else {
                    "fixed-no-projection"
                },
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
        let query_meaning = super::super::semantic_encoding::framed_parts(&[
            self.identity,
            &graph,
            &result_limit,
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
                if self.discovered {
                    "discovered-query-batch"
                } else {
                    "fixed-query-batch"
                },
                &items.to_string(),
                &query_meaning,
            ]),
            None if self.discovered => {
                super::super::semantic_encoding::framed_parts(&["discovered-query", &query_meaning])
            }
            None => query_meaning,
        }
    }
}
