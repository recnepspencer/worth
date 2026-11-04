//! Finite typed owner demands reuse installed program output connections.
use super::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryProducerOutputFamily,
};
use crate::{
    application_operation::ApplicationMutationBinding, application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema, portable_identity::WorthQueryPortableTypeIdentity,
};

type Family<S, C> = <<C as WorthQueryApplicationRequiredOutputConnection<S>>::Demand as super::WorthQueryApplicationOutputDemand<S>>::OutputFamily;
type Source<S, C> = <Family<S, C> as WorthQueryProducerOutputFamily<S>>::Source;

/// A binding names only connections for which it supplies the existing typed demand.
/// Values are derived from its input and retained pre-image during preparation.
/// Coupled demands can move with the mutation request between execution workers.
pub trait ApplicationCoupledOutputVisitor<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn required<Connection>(&mut self)
    where
        Connection: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Connection::Demand: Send,
        Binding: WorthQueryApplicationRequiredOutputSource<Schema, Connection>,
        <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::Input:
            crate::application_query::ApplicationQueryIntent<
                Schema,
                Binding = Source<Schema, Connection>,
            >,
        <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            crate::application_query::ApplicationQueryScopeResolution<
                Schema,
                <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >;
}

/// Portable connection meaning; it carries neither a producer input nor authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredCoupledOutputConnection {
    pub connection: String,
    pub family: String,
    pub source_binding: String,
    pub source_query: WorthQueryPortableTypeIdentity,
    pub source_input: WorthQueryPortableTypeIdentity,
}
impl DeclaredCoupledOutputConnection {
    pub fn of<Schema, Connection>() -> Self
    where
        Schema: ApplicationSchema,
        Connection: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        type Family<S, C> = <<C as WorthQueryApplicationRequiredOutputConnection<S>>::Demand as super::WorthQueryApplicationOutputDemand<S>>::OutputFamily;
        type Source<S, C> = <Family<S, C> as WorthQueryProducerOutputFamily<S>>::Source;
        let source = Source::<Schema, Connection>::descriptor();
        Self {
            connection: Connection::IDENTITY.to_owned(),
            family: Family::<Schema, Connection>::IDENTITY.to_owned(),
            source_binding: source.identity().to_owned(),
            source_query: source.query_identity().clone(),
            source_input: source.input_identity().clone(),
        }
    }
}

pub(crate) struct DescribeCoupledOutputs {
    pub(crate) declarations: Vec<DeclaredCoupledOutputConnection>,
}
impl DescribeCoupledOutputs {
    pub(crate) fn new() -> Self {
        Self {
            declarations: Vec::new(),
        }
    }
    pub(crate) fn finish(mut self) -> Vec<DeclaredCoupledOutputConnection> {
        self.declarations.sort();
        self.declarations
    }
}
impl<Schema, Binding> ApplicationCoupledOutputVisitor<Schema, Binding> for DescribeCoupledOutputs
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn required<Connection>(&mut self)
    where
        Connection: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Connection::Demand: Send,
        Binding: WorthQueryApplicationRequiredOutputSource<Schema, Connection>,
        <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::Input:
            crate::application_query::ApplicationQueryIntent<
                Schema,
                Binding = Source<Schema, Connection>,
            >,
        <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            crate::application_query::ApplicationQueryScopeResolution<
                Schema,
                <Source<Schema, Connection> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        self.declarations
            .push(DeclaredCoupledOutputConnection::of::<Schema, Connection>());
    }
}
