use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationRelationRef};

use super::{
    ApplicationNameDiscriminator, ApplicationNameDiscriminatorDecode,
    ApplicationNameDiscriminatorDecodeDenial, ApplicationNameDiscriminatorDecoder,
    ApplicationNameDiscriminatorEncoder, ApplicationNameSchemeBuilder,
};

/// The roles of one name scheme as a Rust type, so a birth names its role by
/// value and a scheme declares its role order from the type.
///
/// Implement it with `#[derive(WorthQueryNameRole)]` on an enum whose variants
/// are the roles: a unit variant is one role, and a variant carrying one
/// `u32` is one role repeated by a numeric index. `#[depends_on(Variant, ..)]`
/// on a variant declares that its births may reference names born under those
/// roles in the same occurrence. The derive refuses an unknown, repeated or
/// self dependency and any dependency cycle at the attribute, and orders the
/// type by canonical role order and then by index, so `Instance(10)` sorts
/// after `Instance(9)`.
///
/// # How a role reaches a name
///
/// A name is minted from its scheme, occurrence, role path, parents and
/// discriminator, and the role path must be one of the scheme's declared
/// roles. [`Self::role_name`] is therefore the declared role alone, for every
/// index: `Instance(2)` and `Instance(10)` are both born under `Instance`.
/// The index tells the siblings apart as the leading field of the birth's
/// canonical discriminator ([`IndexedRoleDiscriminator`]), where it is
/// compared as a number.
pub trait WorthQueryNameRole: Ord + Sized + 'static {
    /// Every declared role, each before the roles that depend on it.
    const ROLE_ORDER: &'static [&'static str];

    /// `(role, depends_on)`: births under `role` may reference names born
    /// under `depends_on` in the same occurrence.
    const ROLE_DEPENDENCIES: &'static [(&'static str, &'static str)];

    /// The declared role this value is born under; always one of
    /// [`Self::ROLE_ORDER`].
    fn role_name(&self) -> &'static str;

    /// Which repetition of an indexed role this is.
    fn role_index(&self) -> Option<u32> {
        None
    }

    /// The role a recorded birth was born under: the inverse of
    /// [`Self::role_name`] and [`Self::role_index`]. `None` when `role` is
    /// not one of this type's roles, or when its index does not fit it (an
    /// index on a plain role, or none on an indexed one).
    fn from_role_parts(role: &str, index: Option<u32>) -> Option<Self>;
}

/// Identity of the canonical discriminator an indexed role's birth carries
/// when it declares no typed discriminator of its own.
pub const NAME_ROLE_INDEX_IDENTITY: &str = "worth.query.name-role-index.v1";

/// Label of the field an indexed role's index is written under. A Rust field
/// name cannot spell it, so no derived discriminator declares it.
pub(crate) const NAME_ROLE_INDEX_FIELD: &str = "worth.role.index";

impl ApplicationNameDiscriminatorEncoder {
    /// Writes which repetition of an indexed role a birth is. It is the
    /// leading field of that birth's canonical discriminator, ahead of
    /// anything its typed discriminator encodes.
    pub fn role_index(&mut self, index: u32) -> &mut Self {
        self.field(NAME_ROLE_INDEX_FIELD, &index)
    }
}

/// The canonical discriminator of a birth under an indexed role: the role's
/// index, then the birth's own discriminator `D`.
///
/// A birth authored from a typed role carries exactly these bytes, so this
/// is the value that selects it afterwards:
/// `NameSelection::keep_side(&IndexedRoleDiscriminator::new(3, side))` keeps
/// the piece of repetition `3` born on `side`, where `keep_side(&side)` alone
/// matches no indexed piece.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexedRoleDiscriminator<D = IndexedRoleAlone> {
    index: u32,
    discriminator: D,
}

impl<D: ApplicationNameDiscriminator> IndexedRoleDiscriminator<D> {
    pub const fn new(index: u32, discriminator: D) -> Self {
        Self {
            index,
            discriminator,
        }
    }
}

impl IndexedRoleDiscriminator {
    /// The discriminator of a birth told apart by its role index alone.
    pub const fn index_alone(index: u32) -> Self {
        Self::new(index, IndexedRoleAlone)
    }
}

impl<D: ApplicationNameDiscriminator> ApplicationNameDiscriminator for IndexedRoleDiscriminator<D> {
    const DISCRIMINATOR_IDENTITY: &'static str = D::DISCRIMINATOR_IDENTITY;

    fn referenced_roles() -> &'static [&'static str] {
        D::referenced_roles()
    }

    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.role_index(self.index);
        self.discriminator.encode_canonical(encoder);
    }
}

impl<D: ApplicationNameDiscriminatorDecode> ApplicationNameDiscriminatorDecode
    for IndexedRoleDiscriminator<D>
{
    const LEADS_WITH_ROLE_INDEX: bool = true;

    fn decode_canonical(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Result<Self, ApplicationNameDiscriminatorDecodeDenial> {
        let index = decoder.role_index()?;
        Ok(Self::new(index, D::decode_canonical(decoder)?))
    }
}

impl<D> IndexedRoleDiscriminator<D> {
    /// Which repetition of the indexed role the birth is.
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// The birth's own discriminator, after its role index.
    pub const fn discriminator(&self) -> &D {
        &self.discriminator
    }
}

/// The empty discriminator of a birth its role index alone tells apart.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexedRoleAlone;

impl ApplicationNameDiscriminator for IndexedRoleAlone {
    const DISCRIMINATOR_IDENTITY: &'static str = NAME_ROLE_INDEX_IDENTITY;

    fn encode_canonical(&self, _: &mut ApplicationNameDiscriminatorEncoder) {}
}

impl ApplicationNameDiscriminatorDecode for IndexedRoleAlone {
    fn decode_canonical(
        _: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Result<Self, ApplicationNameDiscriminatorDecodeDenial> {
        Ok(Self)
    }
}

impl<S, K> ApplicationNameSchemeBuilder<S, K>
where
    K: ApplicationEntityMarkerIdentity<S>,
{
    /// Starts a scheme for declared entity kind `K` whose role order and role
    /// dependencies are those of `R`.
    pub fn from_roles<R: WorthQueryNameRole>(scheme_id: impl Into<String>) -> Self {
        Self::new(scheme_id).declared_roles::<R>()
    }
}

impl<S, K> ApplicationNameSchemeBuilder<S, K> {
    /// Starts a scheme for a declared relation kind whose role order and role
    /// dependencies are those of `R`.
    pub fn for_relation_from_roles<R: WorthQueryNameRole, From, To>(
        scheme_id: impl Into<String>,
        relation: ApplicationRelationRef<S, K, From, To>,
    ) -> Self {
        Self::for_relation(scheme_id, relation).declared_roles::<R>()
    }

    fn declared_roles<R: WorthQueryNameRole>(self) -> Self {
        R::ROLE_DEPENDENCIES.iter().fold(
            self.role_order(R::ROLE_ORDER),
            |builder, (role, depends_on)| builder.role_depends_on(*role, *depends_on),
        )
    }
}
