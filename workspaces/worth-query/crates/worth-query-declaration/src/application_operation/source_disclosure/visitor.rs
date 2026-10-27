use std::any::TypeId;
use std::marker::PhantomData;

use crate::application_query::ApplicationQueryBinding;
use crate::application_schema::ApplicationSchema;

use super::super::ApplicationMutationBinding;
use super::{ApplicationSourceDisclosureBinding, DeclaredApplicationSourceDisclosure};

/// Typed dispatch from the actual source binding, never an erased callback map.
pub trait ApplicationSourceDisclosureVisitor<
    Schema: ApplicationSchema,
    Source: ApplicationMutationBinding<Schema>,
>: Sized
{
    type Output;
    fn unavailable(self) -> Self::Output;
    fn declared<Binding: ApplicationSourceDisclosureBinding<Schema, Source = Source>>(
        self,
    ) -> Self::Output;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceDisclosureDescriptor {
    pub(crate) declaration: DeclaredApplicationSourceDisclosure,
    pub(crate) query_type: TypeId,
}

pub(crate) struct DescribeSourceDisclosure<Schema, Source>(PhantomData<fn() -> (Schema, Source)>);

impl<Schema, Source> DescribeSourceDisclosure<Schema, Source> {
    pub(crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Schema: ApplicationSchema, Source: ApplicationMutationBinding<Schema>>
    ApplicationSourceDisclosureVisitor<Schema, Source>
    for DescribeSourceDisclosure<Schema, Source>
{
    type Output = Option<SourceDisclosureDescriptor>;

    fn unavailable(self) -> Self::Output {
        None
    }

    fn declared<Binding: ApplicationSourceDisclosureBinding<Schema, Source = Source>>(
        self,
    ) -> Self::Output {
        Some(SourceDisclosureDescriptor {
            declaration: DeclaredApplicationSourceDisclosure::of::<Schema, Binding>(),
            query_type: TypeId::of::<<Binding::Disclosure as ApplicationQueryBinding<Schema>>::Query>(
            ),
        })
    }
}
