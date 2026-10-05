//! Exact initial final-ring outputs and their retained upstream source.
use super::*;

/// The actual upstream source, preserved outside the final ring's generated payload.
pub struct FinalRetainedSourceOutput<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole
    for FinalRetainedSourceOutput<Schema>
{
    type Schema = Schema;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "source";
}

/// One declaration-owned preserved source member selected by its actual key.
pub struct FinalRetainedSources<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRoleFamily
    for FinalRetainedSources<Schema>
{
    type Schema = Schema;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    const PREFIX: &'static str = "source.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::PRESERVE;
    const MINIMUM: usize = 1;
}

pub struct FinalPlanarOutputs;

/// The vertex that anchors the final ring.
pub struct FinalAnchorOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole for FinalAnchorOutput<Schema> {
    type Schema = Schema;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "anchor";
}

/// The vertex that closes the final ring. Declared at-most-one; every final
/// output this fixture publishes binds it.
pub struct FinalClosingOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole for FinalClosingOutput<Schema> {
    type Schema = Schema;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "closing";
}

/// A declared at-most-one role no final output binds.
pub struct FinalAuxiliaryOutput<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRole
    for FinalAuxiliaryOutput<Schema>
{
    type Schema = Schema;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "auxiliary";
}

/// The other vertices the final ring creates, one member per vertex.
pub struct FinalCreatedOutputs<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputRoleFamily
    for FinalCreatedOutputs<Schema>
{
    type Schema = Schema;
    type Contract = FinalPlanarOutputs;
    type Entity = Body;
    const PREFIX: &'static str = "created.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::CREATE;
    const MINIMUM: usize = 0;
}

impl<Schema: TopologySchemaBinding> ApplicationMutationOutputContract<Schema>
    for FinalPlanarOutputs
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <FinalRetainedSourceOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <FinalAnchorOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <FinalClosingOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <FinalAuxiliaryOutput<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] = &[
        <FinalRetainedSources<Schema> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
        <FinalCreatedOutputs<Schema> as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR,
    ];
}
