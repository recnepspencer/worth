//! One-hop physical projection slots, shared by the complete Source profile.
use super::*;
use worth_query_decl::facade::worth_query_portable_type;
pub struct RootXSlot;
worth_query_portable_type!(RootXSlot => "worth.query.scene-face.root_x.v1");
pub(in crate::scene_faces) fn root_x<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    RootXSlot,
    S,
    Body,
    PlanarPosition,
    PositionX,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, ()>,
> {
    ApplicationQueryResultFieldRef::new("root_x", PositionX::reference())
}
pub struct RootYSlot;
worth_query_portable_type!(RootYSlot => "worth.query.scene-face.root_y.v1");
pub(in crate::scene_faces) fn root_y<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    RootYSlot,
    S,
    Body,
    PlanarPosition,
    PositionY,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, ()>,
> {
    ApplicationQueryResultFieldRef::new("root_y", PositionY::reference())
}
pub struct NextXSlot;
worth_query_portable_type!(NextXSlot => "worth.query.scene-face.next_x.v1");
pub(in crate::scene_faces) fn next_x<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    NextXSlot,
    S,
    Body,
    PlanarPosition,
    PositionX,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, ()>,
> {
    ApplicationQueryResultFieldRef::new("next_x", PositionX::reference())
}
pub struct NextYSlot;
worth_query_portable_type!(NextYSlot => "worth.query.scene-face.next_y.v1");
pub(in crate::scene_faces) fn next_y<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    NextYSlot,
    S,
    Body,
    PlanarPosition,
    PositionY,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, ()>,
> {
    ApplicationQueryResultFieldRef::new("next_y", PositionY::reference())
}
pub struct PreviousXSlot;
worth_query_portable_type!(PreviousXSlot => "worth.query.scene-face.previous_x.v1");
pub(in crate::scene_faces) fn previous_x<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    PreviousXSlot,
    S,
    Body,
    PlanarPosition,
    PositionX,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, ()>,
> {
    ApplicationQueryResultFieldRef::new("previous_x", PositionX::reference())
}
pub struct PreviousYSlot;
worth_query_portable_type!(PreviousYSlot => "worth.query.scene-face.previous_y.v1");
pub(in crate::scene_faces) fn previous_y<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    PreviousYSlot,
    S,
    Body,
    PlanarPosition,
    PositionY,
    PositiveLength,
    ReadWrite,
    EqualityPredicate,
    DeclaredApplicationUnit<Metre, ()>,
> {
    ApplicationQueryResultFieldRef::new("previous_y", PositionY::reference())
}
pub struct AnchorKeySlot;
worth_query_portable_type!(AnchorKeySlot => "worth.query.scene-face.anchor_key.v1");
pub(in crate::scene_faces) fn anchor_key<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    AnchorKeySlot,
    S,
    Body,
    PlanarPosition,
    BodyKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("anchor_key", BodyKey::reference())
}
pub struct FaceKeySlot;
worth_query_portable_type!(FaceKeySlot => "worth.query.scene-face.face_key.v1");
pub(in crate::scene_faces) fn face_key<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultFieldRef<
    Q,
    FaceKeySlot,
    S,
    Face,
    FaceFacts,
    FaceKey,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("face_key", FaceKey::reference())
}
pub struct NextSlot;
worth_query_portable_type!(NextSlot => "worth.query.scene-face.next.v1");
pub(in crate::scene_faces) fn next<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultRelationRef<
    Q,
    NextSlot,
    S,
    PlanarSuccessor,
    Body,
    Body,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("next", PlanarSuccessor::reference())
}
pub struct PreviousSlot;
worth_query_portable_type!(PreviousSlot => "worth.query.scene-face.previous.v1");
pub(in crate::scene_faces) fn previous<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultRelationRef<
    Q,
    PreviousSlot,
    S,
    PlanarSuccessor,
    Body,
    Body,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("previous", PlanarSuccessor::reference())
}
pub struct FaceSlot;
worth_query_portable_type!(FaceSlot => "worth.query.scene-face.face.v1");
pub(in crate::scene_faces) fn face<S: SceneFaceSchemaBinding, Q>(
) -> ApplicationQueryResultRelationRef<
    Q,
    FaceSlot,
    S,
    FaceAnchor,
    Face,
    Body,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("face", FaceAnchor::reference())
}
