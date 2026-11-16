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
    demand_only_root: bool,
    occurrence_component: Option<std::any::TypeId>,
    input_capture: Option<(std::any::TypeId, &'static str, std::any::TypeId)>,
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
            demand_only_root: false,
            occurrence_component: None,
            input_capture: None,
        }
    }

    pub(in crate::application_program) fn set_occurrence_component(
        &mut self,
        component: std::any::TypeId,
    ) {
        self.occurrence_component = Some(component);
    }
    /// Nominal point identity for validating complete component placements.
    /// It is neither portable program meaning nor execution authority.
    pub const fn occurrence_component(&self) -> Option<std::any::TypeId> {
        self.occurrence_component
    }
    /// Internal declaration cycles are resolved from complete admitted occurrences.
    pub const fn is_occurrence_component(&self) -> bool {
        self.occurrence_component.is_some()
    }

    pub(in crate::application_program) fn set_input_capture<C: 'static, Port: 'static>(
        &mut self,
        identity: &'static str,
    ) {
        self.input_capture = Some((
            std::any::TypeId::of::<C>(),
            identity,
            std::any::TypeId::of::<Port>(),
        ));
    }

    /// Stable declared meaning; a capture payload itself is never serialized.
    pub fn input_capture_identity(&self) -> Option<&'static str> {
        self.input_capture.map(|(_, identity, _)| identity)
    }

    /// Nominal point equality, not a portable identity or a read capability.
    pub fn input_capture_type(&self) -> Option<std::any::TypeId> {
        self.input_capture.map(|(nominal, _, _)| nominal)
    }

    /// The concrete declared base port; this is point metadata, never wire identity.
    pub fn input_capture_port_type(&self) -> Option<std::any::TypeId> {
        self.input_capture.map(|(_, _, port)| port)
    }

    pub(in crate::application_program) fn as_demand_only_root(mut self) -> Self {
        self.demand_only_root = true;
        self
    }

    /// Whether this connection is the root of an explicitly demanded output tree.
    pub const fn is_demand_only_root(&self) -> bool {
        self.demand_only_root
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
