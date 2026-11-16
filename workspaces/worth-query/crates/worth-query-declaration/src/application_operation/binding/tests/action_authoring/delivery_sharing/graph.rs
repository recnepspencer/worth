//! A delivery's transitive graph remains distinct from its root connection alone.
use super::*;
pub(super) struct Last;
pub(super) struct LastInput;
pub(super) struct NextOutput;
struct Next;
impl ApplicationFeature<MutationSchema> for Last {
    type Inputs = ApplicationFeatureInputList<LastInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "test.action.last.v1";
}
impl ApplicationInputPort<MutationSchema, Last> for LastInput {
    type Value = RenameInputBinding;
    const IDENTITY: &'static str = "test.action.last-input.v1";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<MutationSchema, Target> for NextOutput {
    type Value = RenameInputBinding;
    const IDENTITY: &'static str = "test.action.next-output.v1";
}
impl ApplicationConnectionIdentity for Next {
    const IDENTITY: &'static str = "test.action.next-connection.v1";
}
impl ApplicationOccurrenceConnectionBinding<MutationSchema, Target, Last> for Next {}
type ExtendedRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<MutationSchema, Source, Export, Target, Import, Connection>,
    ApplicationOutputEdge<
        ApplicationConnectionRef<MutationSchema, Target, NextOutput, Last, LastInput, Next>,
        ApplicationOutputLeaf,
    >,
>;
struct Extended;
impl ApplicationActionDelivery<MutationSchema, RenameBinding> for Extended {
    type Mode = ApplicationFixedDeliveryMode<ExtendedRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "test.action.fixed-delivery.v1";
}
impl ApplicationFixedActionDelivery<MutationSchema, RenameBinding> for Extended {
    type Root = ExtendedRoot;
    type Projection = ApplicationNoProjection;
}
#[test]
fn equal_root_connections_do_not_collapse_distinct_transitive_delivery_graphs() {
    let subset = ApplicationProgramAuthoring::<MutationSchema, Single<Delivery>>::begin()
        .validated_program()
        .unwrap();
    let complete = ApplicationProgramAuthoring::<MutationSchema, Single<Extended>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(
        subset.actions()[0]
            .delivery()
            .unwrap()
            .output()
            .unwrap()
            .root_connection(),
        complete.actions()[0]
            .delivery()
            .unwrap()
            .output()
            .unwrap()
            .root_connection()
    );
    assert_eq!(subset.connections().len(), 1);
    assert_eq!(complete.connections().len(), 2);
    let short = subset.normalized_manifest();
    let full = complete.normalized_manifest();
    assert_eq!(records(&short, "delivery-connection-v1").len(), 1);
    assert_eq!(records(&full, "delivery-connection-v1").len(), 2);
    assert_ne!(
        records(&short, "delivery-definition-v1"),
        records(&full, "delivery-definition-v1")
    );
    assert_ne!(subset.revision(), complete.revision());
}
