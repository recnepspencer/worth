use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationFeature, ApplicationInputPort, ApplicationOutputPort, ApplicationPortRef,
    ApplicationProgramConnectionRole,
};

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

    pub(crate) const fn declaration(
        role: ApplicationProgramConnectionRole,
        node_type: std::any::TypeId,
    ) -> ApplicationConnectionDeclaration {
        ApplicationConnectionDeclaration::new(
            <Binding as ApplicationConnectionIdentity>::IDENTITY,
            node_type,
            std::any::TypeId::of::<Binding>(),
            ApplicationConnectionEndpointDeclaration::new(
                SourceFeature::IDENTITY,
                std::any::TypeId::of::<SourceFeature>(),
                SourcePort::IDENTITY,
                std::any::TypeId::of::<SourcePort>(),
            ),
            ApplicationConnectionEndpointDeclaration::new(
                TargetFeature::IDENTITY,
                std::any::TypeId::of::<TargetFeature>(),
                TargetPort::IDENTITY,
                std::any::TypeId::of::<TargetPort>(),
            ),
            role,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ApplicationConnectionEndpointDeclaration {
    feature: &'static str,
    feature_type: std::any::TypeId,
    port: &'static str,
    port_type: std::any::TypeId,
}

impl ApplicationConnectionEndpointDeclaration {
    const fn new(
        feature: &'static str,
        feature_type: std::any::TypeId,
        port: &'static str,
        port_type: std::any::TypeId,
    ) -> Self {
        Self {
            feature,
            feature_type,
            port,
            port_type,
        }
    }
}

/// Erased descriptive connection retained by a validated program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationConnectionDeclaration {
    identity: &'static str,
    node_type: std::any::TypeId,
    binding_type: std::any::TypeId,
    source_feature: &'static str,
    source_feature_type: std::any::TypeId,
    source_port: &'static str,
    source_port_type: std::any::TypeId,
    target_feature: &'static str,
    target_feature_type: std::any::TypeId,
    target_port: &'static str,
    target_port_type: std::any::TypeId,
    role: ApplicationProgramConnectionRole,
}

impl ApplicationConnectionDeclaration {
    const fn new(
        identity: &'static str,
        node_type: std::any::TypeId,
        binding_type: std::any::TypeId,
        source: ApplicationConnectionEndpointDeclaration,
        target: ApplicationConnectionEndpointDeclaration,
        role: ApplicationProgramConnectionRole,
    ) -> Self {
        Self {
            identity,
            node_type,
            binding_type,
            source_feature: source.feature,
            source_feature_type: source.feature_type,
            source_port: source.port,
            source_port_type: source.port_type,
            target_feature: target.feature,
            target_feature_type: target.feature_type,
            target_port: target.port,
            target_port_type: target.port_type,
            role,
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
    pub const fn node_type(&self) -> std::any::TypeId {
        self.node_type
    }
    pub const fn binding_type(&self) -> std::any::TypeId {
        self.binding_type
    }
    pub const fn source_feature_type(&self) -> std::any::TypeId {
        self.source_feature_type
    }
    pub const fn source_port_type(&self) -> std::any::TypeId {
        self.source_port_type
    }
    pub const fn target_feature_type(&self) -> std::any::TypeId {
        self.target_feature_type
    }
    pub const fn target_port_type(&self) -> std::any::TypeId {
        self.target_port_type
    }

    pub const fn role(&self) -> ApplicationProgramConnectionRole {
        self.role
    }
}
