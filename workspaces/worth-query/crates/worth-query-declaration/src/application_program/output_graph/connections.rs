use super::*;
use crate::application_program::ApplicationCompositionInstance;

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    ApplicationConnectionShape<Schema>
    for ApplicationConnectionRef<
        Schema,
        SourceFeature,
        SourcePort,
        TargetFeature,
        TargetPort,
        Binding,
    >
where
    Schema: ApplicationSchema + 'static,
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
    type Binding = Binding;
    type SourceFeature = SourceFeature;
    type SourcePort = SourcePort;
    type TargetFeature = TargetFeature;
    type TargetPort = TargetPort;

    fn declaration() -> ApplicationConnectionDeclaration {
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
    > ApplicationConnectionShape<Schema>
    for ApplicationConnectionInstanceRef<
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
    Schema: ApplicationSchema + 'static,
    SourceInstance: ApplicationCompositionInstance,
    TargetInstance: ApplicationCompositionInstance,
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
    type Binding = Binding;
    type SourceFeature = SourceFeature;
    type SourcePort = SourcePort;
    type TargetFeature = TargetFeature;
    type TargetPort = TargetPort;

    fn declaration() -> ApplicationConnectionDeclaration {
        Self::declaration()
    }
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding> sealed::ConnectionShape
    for ApplicationConnectionRef<
        Schema,
        SourceFeature,
        SourcePort,
        TargetFeature,
        TargetPort,
        Binding,
    >
{
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
    > sealed::ConnectionShape
    for ApplicationConnectionInstanceRef<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    >
{
}
