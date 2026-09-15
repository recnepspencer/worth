use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{ApplicationFeature, ApplicationInputPort, ApplicationOutputPort, ApplicationPortRef};

pub trait ApplicationConnectionIdentity: Sized + 'static {
    const IDENTITY: &'static str;
}

/// Domain-owned mapping between concrete source and target occurrences.
pub trait ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>:
    ApplicationConnectionIdentity
where
    Schema: ApplicationSchema,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
{
}

/// A typed authored connection. Its constructor only exists for equal port values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationConnectionRef<
    Schema,
    SourceFeature,
    SourcePort,
    TargetFeature,
    TargetPort,
    Binding,
> {
    marker: PhantomData<
        fn() -> (
            Schema,
            SourceFeature,
            SourcePort,
            TargetFeature,
            TargetPort,
            Binding,
        ),
    >,
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    ApplicationConnectionRef<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
where
    Schema: ApplicationSchema,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<
        Schema,
        TargetFeature,
        Value = <SourcePort as ApplicationOutputPort<Schema, SourceFeature>>::Value,
    >,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
{
    pub const fn connect(
        _: ApplicationPortRef<Schema, SourceFeature, SourcePort>,
        _: ApplicationPortRef<Schema, TargetFeature, TargetPort>,
    ) -> Self {
        Self {
            marker: PhantomData,
        }
    }

    pub const fn declaration() -> ApplicationConnectionDeclaration {
        ApplicationConnectionDeclaration::new(
            <Binding as ApplicationConnectionIdentity>::IDENTITY,
            SourceFeature::IDENTITY,
            SourcePort::IDENTITY,
            TargetFeature::IDENTITY,
            TargetPort::IDENTITY,
        )
    }

    pub const fn into_declaration(self) -> ApplicationConnectionDeclaration {
        Self::declaration()
    }
}

/// Erased descriptive connection retained by a validated program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationConnectionDeclaration {
    identity: &'static str,
    source_feature: &'static str,
    source_port: &'static str,
    target_feature: &'static str,
    target_port: &'static str,
}

impl ApplicationConnectionDeclaration {
    pub const fn new(
        identity: &'static str,
        source_feature: &'static str,
        source_port: &'static str,
        target_feature: &'static str,
        target_port: &'static str,
    ) -> Self {
        Self {
            identity,
            source_feature,
            source_port,
            target_feature,
            target_port,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn source_feature(&self) -> &'static str {
        self.source_feature
    }
    pub const fn source_port(&self) -> &'static str {
        self.source_port
    }
    pub const fn target_feature(&self) -> &'static str {
        self.target_feature
    }
    pub const fn target_port(&self) -> &'static str {
        self.target_port
    }
}
