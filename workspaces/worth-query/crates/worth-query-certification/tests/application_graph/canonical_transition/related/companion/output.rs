//! Complete fixed-role output contracts share one actual publication.
use super::super::schema::CanonicalSchema;
use super::schema::*;
use worth_query_host::facade::declaration::application_operation::*;
pub struct CompanionOutputs;
pub struct CreatedOutput;
impl WorthQueryApplicationOutputRole for CreatedOutput {
    type Schema = CanonicalSchema;
    type Contract = CompanionOutputs;
    type Entity = Companion;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "companion-left";
}
pub struct CreatedLeft;
impl WorthQueryApplicationOutputRole for CreatedLeft {
    type Schema = CanonicalSchema;
    type Contract = CompanionOutputs;
    type Entity = Companion;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "companion-right";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for CompanionOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        CreatedOutput::DESCRIPTOR,
        CreatedLeft::DESCRIPTOR,
        MixedOutput::DESCRIPTOR,
    ];
}
pub struct PreservedOutputs;
pub struct PreservedOutput;
impl WorthQueryApplicationOutputRole for PreservedOutput {
    type Schema = CanonicalSchema;
    type Contract = PreservedOutputs;
    type Entity = Companion;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "companion-left";
}
pub struct PreservedLeft;
impl WorthQueryApplicationOutputRole for PreservedLeft {
    type Schema = CanonicalSchema;
    type Contract = PreservedOutputs;
    type Entity = Companion;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "companion-right";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for PreservedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[PreservedOutput::DESCRIPTOR, PreservedLeft::DESCRIPTOR];
}

pub const HANDLER_IDENTITY: &str = "canonical-companion.handler.v1";
pub const PROVIDER_IDENTITY: &str = "canonical-companion.provider.v1";
pub const PRESERVE_HANDLER_IDENTITY: &str = "canonical-companion.preserve.handler.v1";
pub const PRESERVE_PROVIDER_IDENTITY: &str = "canonical-companion.preserve.provider.v1";

pub struct MixedOutput;
impl WorthQueryApplicationOutputRole for MixedOutput {
    type Schema = CanonicalSchema;
    type Contract = CompanionOutputs;
    type Entity = super::super::schema::Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "mixed-derived";
}
