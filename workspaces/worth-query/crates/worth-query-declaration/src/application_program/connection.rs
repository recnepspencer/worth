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
    /// Explicitly authorizes this port mapping across composition instances.
    const EXPORTS_ACROSS_INSTANCES: bool = false;
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

/// A typed connection qualified by its source and target composition instances.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationConnectionInstanceRef<
    Schema,
    SourceInstance,
    SourceFeature,
    SourcePort,
    TargetInstance,
    TargetFeature,
    TargetPort,
    Binding,
> {
    marker: PhantomData<
        fn() -> (
            Schema,
            SourceInstance,
            SourceFeature,
            SourcePort,
            TargetInstance,
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
            <super::ApplicationRootComposition as super::ApplicationCompositionInstance>::PATH,
            SourceFeature::IDENTITY,
            SourcePort::IDENTITY,
            <super::ApplicationRootComposition as super::ApplicationCompositionInstance>::PATH,
            TargetFeature::IDENTITY,
            TargetPort::IDENTITY,
            TargetPort::REQUIRED,
            Binding::EXPORTS_ACROSS_INSTANCES,
        )
    }

    pub const fn into_declaration(self) -> ApplicationConnectionDeclaration {
        Self::declaration()
    }
}

impl<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    >
    ApplicationConnectionInstanceRef<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    >
where
    Schema: ApplicationSchema,
    SourceInstance: super::ApplicationCompositionInstance,
    TargetInstance: super::ApplicationCompositionInstance,
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
    pub const fn declaration() -> ApplicationConnectionDeclaration {
        ApplicationConnectionDeclaration::new(
            <Binding as ApplicationConnectionIdentity>::IDENTITY,
            SourceInstance::PATH,
            SourceFeature::IDENTITY,
            SourcePort::IDENTITY,
            TargetInstance::PATH,
            TargetFeature::IDENTITY,
            TargetPort::IDENTITY,
            TargetPort::REQUIRED,
            Binding::EXPORTS_ACROSS_INSTANCES,
        )
    }
}

/// Erased descriptive connection retained by a validated program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationConnectionDeclaration {
    identity: &'static str,
    source_instance: &'static str,
    source_feature: &'static str,
    source_port: &'static str,
    target_instance: &'static str,
    target_feature: &'static str,
    target_port: &'static str,
    target_required: bool,
    exports_across_instances: bool,
}

impl ApplicationConnectionDeclaration {
    pub const fn new(
        identity: &'static str,
        source_instance: &'static str,
        source_feature: &'static str,
        source_port: &'static str,
        target_instance: &'static str,
        target_feature: &'static str,
        target_port: &'static str,
        target_required: bool,
        exports_across_instances: bool,
    ) -> Self {
        Self {
            identity,
            source_instance,
            source_feature,
            source_port,
            target_instance,
            target_feature,
            target_port,
            target_required,
            exports_across_instances,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn source_instance(&self) -> &'static str {
        self.source_instance
    }
    pub const fn source_feature(&self) -> &'static str {
        self.source_feature
    }
    pub const fn source_port(&self) -> &'static str {
        self.source_port
    }
    pub const fn target_instance(&self) -> &'static str {
        self.target_instance
    }
    pub const fn target_feature(&self) -> &'static str {
        self.target_feature
    }
    pub const fn target_port(&self) -> &'static str {
        self.target_port
    }
    pub const fn target_required(&self) -> bool {
        self.target_required
    }
    pub const fn exports_across_instances(&self) -> bool {
        self.exports_across_instances
    }
}
