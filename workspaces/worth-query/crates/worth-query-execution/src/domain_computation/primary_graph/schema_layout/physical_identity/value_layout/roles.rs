//! Exact native role inventories for the physical allocation value courts.
use worth_query_declaration::facade::application_schema::fragment::{
    Entity, Fragment, NoBindings, Relation,
};
pub(super) struct Zulu<const EXTRA_EDGE: bool, const ORBIT: bool>;
pub(super) struct Lookalike;
pub(super) struct Earlier;
impl<const ORBIT: bool> Entity for Zulu<false, ORBIT> {
    const NAME: &'static str = "Zulu";
    type Aspects = ();
    type Relations = (Next<false, ORBIT>,);
}
impl<const ORBIT: bool> Entity for Zulu<true, ORBIT> {
    const NAME: &'static str = "Zulu";
    type Aspects = ();
    type Relations = (Next<true, ORBIT>, EarlierEdge<ORBIT>);
}
impl Entity for Lookalike {
    const NAME: &'static str = "worth-query-provider-idempotency";
    type Aspects = ();
    type Relations = ();
}
impl Entity for Earlier {
    const NAME: &'static str = "Earlier";
    type Aspects = ();
    type Relations = ();
}
pub(super) struct Next<const EXTRA_EDGE: bool, const ORBIT: bool>;
impl<const EXTRA_EDGE: bool, const ORBIT: bool> Relation for Next<EXTRA_EDGE, ORBIT>
where
    Zulu<EXTRA_EDGE, ORBIT>: Entity,
{
    const NAME: &'static str = "Next";
    type To = Zulu<EXTRA_EDGE, ORBIT>;
    const ORBIT: bool = ORBIT;
}
pub(super) struct EarlierEdge<const ORBIT: bool>;
impl<const ORBIT: bool> Relation for EarlierEdge<ORBIT> {
    const NAME: &'static str = "EarlierEdge";
    type To = Zulu<true, ORBIT>;
    const ORBIT: bool = ORBIT;
}
pub(super) struct BaseRoles;
impl Fragment for BaseRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<false, false>, Lookalike);
}
pub(super) struct EarlierRoles;
impl Fragment for EarlierRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<false, false>, Lookalike, Earlier);
}
pub(super) struct ExtraEdgeRoles;
impl Fragment for ExtraEdgeRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<true, false>, Lookalike);
}
pub(super) struct ExtendedRoles;
impl Fragment for ExtendedRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<true, false>, Lookalike, Earlier);
}
pub(super) struct OrbitRoles;
impl Fragment for OrbitRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<false, true>, Lookalike);
}
pub(super) struct EarlierOrbitRoles;
impl Fragment for EarlierOrbitRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<false, true>, Lookalike, Earlier);
}
pub(super) struct ExtraEdgeOrbitRoles;
impl Fragment for ExtraEdgeOrbitRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<true, true>, Lookalike);
}
pub(super) struct ExtendedOrbitRoles;
impl Fragment for ExtendedOrbitRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (Zulu<true, true>, Lookalike, Earlier);
}
pub(super) struct SameNamedRoles;
pub(super) struct SameNamedZulu;
pub(super) struct SameNamedEdge;
impl Entity for SameNamedZulu {
    const NAME: &'static str = "Zulu";
    type Aspects = ();
    type Relations = (SameNamedEdge,);
}
impl Relation for SameNamedEdge {
    const NAME: &'static str = "Zulu";
    type To = SameNamedZulu;
}
impl Fragment for SameNamedRoles {
    const NAME: &'static str = "RoleAllocation";
    type Bindings = NoBindings;
    type Members = (SameNamedZulu,);
}
