//! A scenario's typed roles and fields share the actual declared native entity.
use super::*;
use worth_foundational::facade::ContentRefId;
use worth_query_decl::facade::application_schema::fragment::{self, RecordSelection};
pub struct Profile<const CONTENT: bool>;
pub trait NodeProfile<S>: 'static {
    type Node: fragment::Entity
        + Copy
        + OperationCreates<lifecycle::CreateOp>
        + OperationReads<lifecycle::CreateOp>
        + OperationCreates<lifecycle::RestoreOp>
        + OperationReads<lifecycle::RestoreOp>
        + OperationDeletes<lifecycle::RetireOp>
        + OperationReads<lifecycle::RetireOp>
        + OperationCreates<lifecycle::PlainOp>
        + OperationDeletes<lifecycle::PlainOp>
        + OperationReads<lifecycle::PlainOp>
        + OperationCreates<producer::Operation>
        + OperationReads<coupled::producer::Operation>
        + OperationCreates<coupled::producer::Operation>
        + OperationDeletes<coupled::producer::Operation>
        + OperationReads<parameter_inverse::EditOp>
        + OperationReads<projected_memberships::inspection::Operation>
        + OperationReads<relation_splice::inspection::InspectStageLinks>;
    type Id: RecordSelection<
        Entity = Self::Node,
        Aspect = Facts,
        Value = u64,
        Binding = U64ApplicationValueBinding,
    >;
    type Value: RecordSelection<
        Entity = Self::Node,
        Aspect = Values,
        Value = u64,
        Binding = U64ApplicationValueBinding,
    >;
    type Word: RecordSelection<
        Entity = Self::Node,
        Aspect = content::Payload,
        Value = ContentRefId,
        Binding = ContentRefApplicationValueBinding,
    >;
    fn entity() -> ApplicationEntityRef<S, Self::Node>;
    fn id() -> Self::Id;
    fn value() -> Self::Value;
    fn word() -> Option<Self::Word>;
    fn content() -> Option<ApplicationContentFieldRef<Self::Word, content::Format>> {
        Self::word().map(|word| content_field(word, content::Format))
    }
}
macro_rules! profile {
    ($content:literal, $word:expr) => {
        impl<S> NodeProfile<S> for Profile<$content> {
            type Node = NativeNode<$content>;
            type Id = RawId<$content>;
            type Value = RawValue<$content>;
            type Word = content::RawWord<$content>;
            fn entity() -> ApplicationEntityRef<S, Self::Node> {
                fragment::entity(OwnershipModel::<$content>, NativeNode::<$content>)
            }
            fn id() -> Self::Id {
                RawId::<$content>::at("Id")
            }
            fn value() -> Self::Value {
                RawValue::<$content>::at("Value")
            }
            fn word() -> Option<Self::Word> {
                $word
            }
        }
    };
}
profile!(false, None);
profile!(true, Some(content::RawWord::<true>::at("Word")));
pub type Node<S> = <<S as Scenario>::NodeProfile as NodeProfile<S>>::Node;
pub type Id<S> = <<S as Scenario>::NodeProfile as NodeProfile<S>>::Id;
pub type Value<S> = <<S as Scenario>::NodeProfile as NodeProfile<S>>::Value;
pub fn node<S: Scenario>() -> ApplicationEntityRef<S, Node<S>> {
    S::NodeProfile::entity()
}
pub fn id<S: Scenario>() -> Id<S> {
    S::NodeProfile::id()
}
pub fn value<S: Scenario>() -> Value<S> {
    S::NodeProfile::value()
}
