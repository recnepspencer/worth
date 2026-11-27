use std::fmt;

use super::discriminator::sealed;
use super::{
    ApplicationNameDiscriminatorEncoder, ApplicationNameDiscriminatorValue, WorthQueryName,
    WorthQuerySameCommitName,
};

/// A reference to the name of a record of kind `K`, whether that record was
/// named by an earlier commit or is being named by this one.
///
/// A discriminator declares its reference fields once as `NameRef<K>`. Both
/// forms take the wire form of the name they stand for, so the name a
/// discriminator contributes to does not depend on when the referenced record
/// was born.
///
/// Both sources convert with `into()`:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameDiscriminatorEncoder, NameRef, WorthQueryName, WorthQuerySameCommitName,
/// };
///
/// struct Face;
///
/// struct CutBy {
///     cutter: NameRef<Face>,
/// }
///
/// fn by_committed(cutter: WorthQueryName<Face>) -> CutBy {
///     CutBy { cutter: cutter.into() }
/// }
/// fn by_same_commit(cutter: WorthQuerySameCommitName<Face>) -> CutBy {
///     CutBy { cutter: cutter.into() }
/// }
/// fn encode(cut: &CutBy) -> Vec<u8> {
///     let mut encoder = ApplicationNameDiscriminatorEncoder::new("example.cut-by");
///     encoder.field("cutter", &cut.cutter);
///     encoder.finish()
/// }
///
/// let _check: (fn(WorthQueryName<Face>) -> CutBy, fn(WorthQuerySameCommitName<Face>) -> CutBy) =
///     (by_committed, by_same_commit);
/// let _encode: fn(&CutBy) -> Vec<u8> = encode;
/// ```
///
/// A reference to another kind does not fit:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::NameRef;
///
/// struct Face;
/// struct Edge;
///
/// fn require_face(_cutter: NameRef<Face>) {}
///
/// fn misuse(edge: NameRef<Edge>) {
///     require_face(edge);
/// }
/// ```
///
/// Nor does a name of another kind convert into one:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_naming::{NameRef, WorthQueryName};
///
/// struct Face;
/// struct Edge;
///
/// fn misuse(edge: WorthQueryName<Edge>) -> NameRef<Face> {
///     edge.into()
/// }
/// ```
pub enum NameRef<K> {
    /// A name an earlier commit recorded on this branch: live at this
    /// commit's basis, retired by it (a consumed cutter) or retired before
    /// it (a retired ancestor), always by its fixed 32-byte identity.
    Committed(WorthQueryName<K>),
    /// The name a record created and named by this commit will be born with.
    InCommit(WorthQuerySameCommitName<K>),
}

impl<K> From<WorthQueryName<K>> for NameRef<K> {
    fn from(name: WorthQueryName<K>) -> Self {
        Self::Committed(name)
    }
}

impl<K> From<WorthQuerySameCommitName<K>> for NameRef<K> {
    fn from(name: WorthQuerySameCommitName<K>) -> Self {
        Self::InCommit(name)
    }
}

impl<K> Clone for NameRef<K> {
    fn clone(&self) -> Self {
        match self {
            Self::Committed(name) => Self::Committed(*name),
            Self::InCommit(name) => Self::InCommit(name.clone()),
        }
    }
}

/// A committed reference never equals a same-commit one: which name the
/// latter stands for is not known until the commit mints it.
impl<K> PartialEq for NameRef<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Committed(a), Self::Committed(b)) => a == b,
            (Self::InCommit(a), Self::InCommit(b)) => a == b,
            _ => false,
        }
    }
}

impl<K> Eq for NameRef<K> {}

impl<K> fmt::Debug for NameRef<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Committed(name) => formatter.debug_tuple("Committed").field(name).finish(),
            Self::InCommit(name) => formatter.debug_tuple("InCommit").field(name).finish(),
        }
    }
}

impl<K> sealed::Sealed for NameRef<K> {}
impl<K> ApplicationNameDiscriminatorValue for NameRef<K> {
    const MAX_ENCODED_BYTES: Option<usize> = Some(33);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        match self {
            Self::Committed(name) => name.encode_discriminator_value(encoder),
            Self::InCommit(name) => name.encode_discriminator_value(encoder),
        }
    }
}
