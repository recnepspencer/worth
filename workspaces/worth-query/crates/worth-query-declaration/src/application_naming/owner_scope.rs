//! The entity that owns a named record, declared as a chain of relation hops.
//!
//! A consumer declares the chain once, in types, on the kind's
//! [`super::WorthQueryNamedKind`]. Owner-scoped resolution and the owners an
//! ambiguous resolution reports both walk exactly these hops; no owner is
//! ever inferred from a name's text.

use std::marker::PhantomData;

use crate::application_schema::fragment;

/// Which way one hop crosses its relation, walking from the named record
/// toward its owner.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NameOwnerHopDirection {
    /// From the relation's declaring entity to its target.
    Forward,
    /// From the relation's target back to its declaring entity.
    Reverse,
}

/// One declared hop of an owner chain: a relation, its declared endpoints,
/// and the way the chain crosses it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NameOwnerHop {
    relation: &'static str,
    from: &'static str,
    to: &'static str,
    direction: NameOwnerHopDirection,
}

impl NameOwnerHop {
    /// The relation's declared name.
    pub const fn relation(&self) -> &'static str {
        self.relation
    }

    /// The relation's declaring (source) entity.
    pub const fn from(&self) -> &'static str {
        self.from
    }

    /// The relation's target entity.
    pub const fn to(&self) -> &'static str {
        self.to
    }

    pub const fn direction(&self) -> NameOwnerHopDirection {
        self.direction
    }

    /// The entity the hop starts at, on the named record's side.
    pub const fn near(&self) -> &'static str {
        match self.direction {
            NameOwnerHopDirection::Forward => self.from,
            NameOwnerHopDirection::Reverse => self.to,
        }
    }

    /// The entity the hop arrives at, on the owner's side.
    pub const fn far(&self) -> &'static str {
        match self.direction {
            NameOwnerHopDirection::Forward => self.to,
            NameOwnerHopDirection::Reverse => self.from,
        }
    }
}

/// The owner declaration of kind `K`: the owning entity kind and the hops
/// that reach it from a named `K`.
pub trait NameOwnerScope<K> {
    /// The owning entity kind; [`NoNameOwner`] when the kind is unowned.
    type Owner: 'static;
    /// The declared hops in walking order; empty when the kind is unowned.
    fn hops() -> Vec<NameOwnerHop>;
}

/// The owner of a kind that declares none. It has no values, so a list of
/// such owners is always empty.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NoNameOwner {}

/// Declares that no entity owns the kind's named records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Unowned;

impl<K> NameOwnerScope<K> for Unowned {
    type Owner = NoNameOwner;
    fn hops() -> Vec<NameOwnerHop> {
        Vec::new()
    }
}

/// Declares that each named record is owned by the entity `Path` reaches.
///
/// `Path` is one hop ([`OwnerVia`] or [`OwnerViaReverse`]) or a tuple of up
/// to four hops, each starting where the previous one arrived:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     NameOwnerHopDirection, NameOwnerScope, OwnedThrough, OwnerVia, OwnerViaReverse,
/// };
/// use worth_query_declaration::facade::application_schema::fragment::{Entity, Relation};
///
/// struct Body;
/// struct Feature;
/// struct Face;
/// struct IntroducedBy;
/// struct Features;
/// impl Entity for Body {
///     const NAME: &'static str = "Body";
///     type Aspects = ();
///     type Relations = (Features,);
/// }
/// impl Entity for Feature {
///     const NAME: &'static str = "Feature";
///     type Aspects = ();
///     type Relations = ();
/// }
/// impl Entity for Face {
///     const NAME: &'static str = "Face";
///     type Aspects = ();
///     type Relations = (IntroducedBy,);
/// }
/// impl Relation for IntroducedBy {
///     const NAME: &'static str = "IntroducedBy";
///     type To = Feature;
/// }
/// impl Relation for Features {
///     const NAME: &'static str = "Features";
///     type To = Feature;
/// }
///
/// // face -> introducing feature -> the body that lists the feature.
/// type FaceOwner = OwnedThrough<(OwnerVia<IntroducedBy>, OwnerViaReverse<Features, Body>)>;
/// let hops = <FaceOwner as NameOwnerScope<Face>>::hops();
/// assert_eq!((hops[0].near(), hops[0].far()), ("Face", "Feature"));
/// assert_eq!((hops[1].near(), hops[1].far()), ("Feature", "Body"));
/// assert_eq!(hops[1].direction(), NameOwnerHopDirection::Reverse);
/// let _owner: Option<<FaceOwner as NameOwnerScope<Face>>::Owner> = None::<Body>;
/// ```
///
/// A hop that does not start where the previous one arrived fails to compile:
///
/// ```compile_fail,E0271
/// use worth_query_declaration::facade::application_naming::{
///     NameOwnerScope, OwnedThrough, OwnerViaReverse,
/// };
/// use worth_query_declaration::facade::application_schema::fragment::{Entity, Relation};
///
/// struct Body;
/// struct Feature;
/// struct Face;
/// struct Features;
/// impl Entity for Body {
///     const NAME: &'static str = "Body";
///     type Aspects = ();
///     type Relations = (Features,);
/// }
/// impl Entity for Feature {
///     const NAME: &'static str = "Feature";
///     type Aspects = ();
///     type Relations = ();
/// }
/// impl Entity for Face {
///     const NAME: &'static str = "Face";
///     type Aspects = ();
///     type Relations = ();
/// }
/// impl Relation for Features {
///     const NAME: &'static str = "Features";
///     type To = Feature;
/// }
///
/// // `Features` targets `Feature`, not `Face`: it cannot be walked back from a face.
/// let _ = <OwnedThrough<OwnerViaReverse<Features, Body>> as NameOwnerScope<Face>>::hops();
/// ```
pub struct OwnedThrough<Path>(PhantomData<fn() -> Path>);

impl<K: fragment::Entity, Path: NameOwnerPath<K>> NameOwnerScope<K> for OwnedThrough<Path> {
    type Owner = Path::Owner;
    fn hops() -> Vec<NameOwnerHop> {
        let mut hops = Vec::new();
        Path::append_hops(&mut hops);
        hops
    }
}

/// One hop starting at entity `From`.
pub trait NameOwnerStep<From> {
    /// The entity the hop arrives at.
    type To: fragment::Entity;
    fn hop() -> NameOwnerHop;
}

/// A hop along relation `R`, from the entity that declares it to `R::To`.
pub struct OwnerVia<R>(PhantomData<fn() -> R>);

impl<From: fragment::Entity, R: fragment::Relation> NameOwnerStep<From> for OwnerVia<R> {
    type To = R::To;
    fn hop() -> NameOwnerHop {
        NameOwnerHop {
            relation: R::NAME,
            from: From::NAME,
            to: <R::To as fragment::Entity>::NAME,
            direction: NameOwnerHopDirection::Forward,
        }
    }
}

/// A hop back along relation `R`, from its target to `Source`, the entity
/// that declares `R`.
pub struct OwnerViaReverse<R, Source>(PhantomData<fn() -> (R, Source)>);

impl<From, R, Source> NameOwnerStep<From> for OwnerViaReverse<R, Source>
where
    From: fragment::Entity,
    R: fragment::Relation<To = From>,
    Source: fragment::Entity,
{
    type To = Source;
    fn hop() -> NameOwnerHop {
        NameOwnerHop {
            relation: R::NAME,
            from: Source::NAME,
            to: From::NAME,
            direction: NameOwnerHopDirection::Reverse,
        }
    }
}

/// A chain of hops from entity `From` to its owner.
pub trait NameOwnerPath<From> {
    type Owner: fragment::Entity;
    fn append_hops(hops: &mut Vec<NameOwnerHop>);
}

impl<From: fragment::Entity, R: fragment::Relation> NameOwnerPath<From> for OwnerVia<R> {
    type Owner = R::To;
    fn append_hops(hops: &mut Vec<NameOwnerHop>) {
        hops.push(<Self as NameOwnerStep<From>>::hop());
    }
}

impl<From, R, Source> NameOwnerPath<From> for OwnerViaReverse<R, Source>
where
    From: fragment::Entity,
    R: fragment::Relation<To = From>,
    Source: fragment::Entity,
{
    type Owner = Source;
    fn append_hops(hops: &mut Vec<NameOwnerHop>) {
        hops.push(<Self as NameOwnerStep<From>>::hop());
    }
}

impl<From, A, B> NameOwnerPath<From> for (A, B)
where
    A: NameOwnerStep<From>,
    B: NameOwnerStep<A::To>,
{
    type Owner = B::To;
    fn append_hops(hops: &mut Vec<NameOwnerHop>) {
        hops.extend([A::hop(), B::hop()]);
    }
}

impl<From, A, B, C> NameOwnerPath<From> for (A, B, C)
where
    A: NameOwnerStep<From>,
    B: NameOwnerStep<A::To>,
    C: NameOwnerStep<B::To>,
{
    type Owner = C::To;
    fn append_hops(hops: &mut Vec<NameOwnerHop>) {
        hops.extend([A::hop(), B::hop(), C::hop()]);
    }
}

impl<From, A, B, C, D> NameOwnerPath<From> for (A, B, C, D)
where
    A: NameOwnerStep<From>,
    B: NameOwnerStep<A::To>,
    C: NameOwnerStep<B::To>,
    D: NameOwnerStep<C::To>,
{
    type Owner = D::To;
    fn append_hops(hops: &mut Vec<NameOwnerHop>) {
        hops.extend([A::hop(), B::hop(), C::hop(), D::hop()]);
    }
}
