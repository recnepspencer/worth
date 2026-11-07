//! The same portable records appear in owned, extended, payload and exclusive positions.
use super::*;
pub struct Schema;
impl ApplicationSchema for Schema {
    const NAME: &'static str = "revisions";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(
            Root::<Standard, Members<Standard, Row>>(PhantomData),
        )?
        .build()
    }
}
pub struct Root<Set, M>(pub PhantomData<(Set, M)>);
impl<Set: BindingSet, M: 'static> Fragment for Root<Set, M> {
    const NAME: &'static str = "revisions";
    type Bindings = Set;
    type Members = M;
}
pub struct Owner<Set, E>(pub PhantomData<(Set, E)>);
impl<Set: BindingSet, E: Entity> Fragment for Owner<Set, E> {
    const NAME: &'static str = "owner";
    type Bindings = Set;
    type Members = (E, Target);
}
pub struct Extensions<Set, E, A, P>(pub PhantomData<(Set, E, A, P)>);
impl<Set: BindingSet, E: Entity, A: Aspect, P: 'static> Fragment for Extensions<Set, E, A, P> {
    const NAME: &'static str = "extensions";
    type Bindings = Set;
    type Members = (Extend<E, (Option<A>,), (WithAspects<ExtendedLink, P>,)>,);
}
pub struct Item<A, P>(pub PhantomData<(A, P)>);
impl<A: 'static, P: 'static> Entity for Item<A, P> {
    const NAME: &'static str = "item";
    type Aspects = A;
    type Relations = (WithAspects<OwnedLink, P>,);
}
pub struct Target;
impl Entity for Target {
    const NAME: &'static str = "target";
    type Aspects = ();
    type Relations = ();
}
pub struct OwnedLink;
pub struct ExtendedLink;
impl Relation for OwnedLink {
    const NAME: &'static str = "owned_link";
    type To = Target;
}
impl Relation for ExtendedLink {
    const NAME: &'static str = "extended_link";
    type To = Target;
}
macro_rules! aspect_type {
    ($($marker:ident => $name:literal),+) => { $(
        pub struct $marker<R>(pub PhantomData<R>);
        impl<R: 'static> Aspect for $marker<R> { const NAME: &'static str = $name; type Row = R; }
    )+ };
}
aspect_type!(Facts => "facts", ExtraFacts => "extra_facts", Payload => "payload", Case => "case",
    RenamedFacts => "facts", RenamedExtra => "extra_facts", RenamedPayload => "payload", RenamedCase => "case");
pub struct OtherCase;
impl Aspect for OtherCase {
    const NAME: &'static str = "other_case";
    type Row = OtherRow;
}
pub struct Unrelated;
impl Aspect for Unrelated {
    const NAME: &'static str = "unrelated";
    type Row = OtherRow;
}
pub struct Choice<R, C>(PhantomData<(R, C)>);
impl<R: 'static, C: 'static> OneOf for Choice<R, C> {
    type Row = ChoiceRow<R>;
    type Cases = C;
}
pub type Model<R> = Item<(Facts<R>, Exclusive<Choice<R, (Case<R>,)>>), (Payload<R>,)>;
pub type Extras<S, E, R> = Extensions<S, E, ExtraFacts<R>, (Payload<R>,)>;
pub type Members<S, R> = (Owner<S, Model<R>>, Extras<S, Model<R>, R>);
