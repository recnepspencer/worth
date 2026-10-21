use super::*;
pub(super) struct BirthOutputs;
pub(super) struct KeepOutputs;
macro_rules! role {
    ($name:ident, $contract:ty, $entity:ty, $action:ty, $label:literal) => {
        pub(super) struct $name<Schema>(PhantomData<fn() -> Schema>);
        impl<Schema: BirthSchema> WorthQueryApplicationOutputRole for $name<Schema> {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = $entity;
            type Action = $action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $label;
        }
    };
}
role!(
    BornNode,
    BirthOutputs,
    OutputNode,
    WorthQueryCreateOutput,
    "node"
);
role!(
    BornPresent,
    BirthOutputs,
    PresentNode,
    WorthQueryCreateOutput,
    "present"
);
role!(
    BornFieldless,
    BirthOutputs,
    FieldlessNode,
    WorthQueryCreateOutput,
    "fieldless"
);
role!(
    BornStrict,
    BirthOutputs,
    StrictNode,
    WorthQueryCreateOutput,
    "strict"
);
role!(
    KeptNode,
    KeepOutputs,
    OutputNode,
    WorthQueryPreserveOutput,
    "node"
);
role!(
    KeptPresent,
    KeepOutputs,
    PresentNode,
    WorthQueryPreserveOutput,
    "present"
);
role!(
    KeptFieldless,
    KeepOutputs,
    FieldlessNode,
    WorthQueryPreserveOutput,
    "fieldless"
);
role!(
    KeptStrict,
    KeepOutputs,
    StrictNode,
    WorthQueryPreserveOutput,
    "strict"
);
impl<Schema: BirthSchema> ApplicationMutationOutputContract<Schema> for BirthOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <BornNode<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <BornPresent<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <BornFieldless<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <BornStrict<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
impl<Schema: BirthSchema> ApplicationMutationOutputContract<Schema> for KeepOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <KeptNode<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <KeptPresent<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <KeptFieldless<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <KeptStrict<Schema> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}
