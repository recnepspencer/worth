//! Complete fixed-role output contracts share one actual publication.
use super::super::schema::*;
use worth_query_host::facade::declaration::application_operation::*;
pub struct DerivedOutputs;
pub struct CreatedOutput;
impl WorthQueryApplicationOutputRole for CreatedOutput {
    type Schema = CanonicalSchema;
    type Contract = DerivedOutputs;
    type Entity = Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-report";
}
pub struct CreatedLeft;
impl WorthQueryApplicationOutputRole for CreatedLeft {
    type Schema = CanonicalSchema;
    type Contract = DerivedOutputs;
    type Entity = Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-left";
}
pub struct CreatedRight;
impl WorthQueryApplicationOutputRole for CreatedRight {
    type Schema = CanonicalSchema;
    type Contract = DerivedOutputs;
    type Entity = Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-right";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for DerivedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        CreatedOutput::DESCRIPTOR,
        CreatedLeft::DESCRIPTOR,
        CreatedRight::DESCRIPTOR,
    ];
}
pub struct PreservedOutputs;
pub struct PreservedOutput;
impl WorthQueryApplicationOutputRole for PreservedOutput {
    type Schema = CanonicalSchema;
    type Contract = PreservedOutputs;
    type Entity = Derived;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-report";
}
pub struct PreservedLeft;
impl WorthQueryApplicationOutputRole for PreservedLeft {
    type Schema = CanonicalSchema;
    type Contract = PreservedOutputs;
    type Entity = Derived;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-left";
}
pub struct PreservedRight;
impl WorthQueryApplicationOutputRole for PreservedRight {
    type Schema = CanonicalSchema;
    type Contract = PreservedOutputs;
    type Entity = Derived;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-right";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for PreservedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        PreservedOutput::DESCRIPTOR,
        PreservedLeft::DESCRIPTOR,
        PreservedRight::DESCRIPTOR,
    ];
}
pub struct RetiredOutputs;
pub struct RetiredOutput;
impl WorthQueryApplicationOutputRole for RetiredOutput {
    type Schema = CanonicalSchema;
    type Contract = RetiredOutputs;
    type Entity = Derived;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-report";
}
pub struct RetiredLeft;
impl WorthQueryApplicationOutputRole for RetiredLeft {
    type Schema = CanonicalSchema;
    type Contract = RetiredOutputs;
    type Entity = Derived;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-left";
}
pub struct RetiredRight;
impl WorthQueryApplicationOutputRole for RetiredRight {
    type Schema = CanonicalSchema;
    type Contract = RetiredOutputs;
    type Entity = Derived;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-right";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for RetiredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        RetiredOutput::DESCRIPTOR,
        RetiredLeft::DESCRIPTOR,
        RetiredRight::DESCRIPTOR,
    ];
}
