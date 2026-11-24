use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// The 32 bytes a discriminator carries where a same-commit name will be
/// written once it is known. They never name anything: a discriminator
/// committed with them still in place is refused as an undeclared reference.
pub const SAME_COMMIT_NAME_PLACEHOLDER: [u8; 32] = [0u8; 32];

/// Kind-typed reference to the name a record of kind `K`, created and named
/// by the same candidate, will be born with.
///
/// It is not a name: nothing resolves it, and it reveals no digest. Its only
/// use is as a discriminator value of a later-role output of the same
/// occurrence in the same candidate, where WORTH writes the referenced name
/// at commit. The candidate writer that named the output is its only source.
///
/// A handler cannot construct one:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::WorthQuerySameCommitName;
///
/// struct Face;
///
/// let _forged = WorthQuerySameCommitName::<Face>::new(0);
/// ```
///
/// A reference to another kind does not fit a discriminator field:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::WorthQuerySameCommitName;
///
/// struct Face;
/// struct Edge;
///
/// struct CutBy {
///     cutter: WorthQuerySameCommitName<Face>,
/// }
///
/// fn misuse(edge: WorthQuerySameCommitName<Edge>) -> CutBy {
///     CutBy { cutter: edge }
/// }
/// ```
///
/// The same-kind reference encodes as a discriminator value:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameDiscriminatorEncoder, WorthQuerySameCommitName,
/// };
///
/// struct Face;
///
/// fn encode(cutter: &WorthQuerySameCommitName<Face>) -> Vec<u8> {
///     let mut encoder = ApplicationNameDiscriminatorEncoder::new("example.cut-by");
///     encoder.field("cutter", cutter);
///     encoder.finish()
/// }
///
/// let _check: fn(&WorthQuerySameCommitName<Face>) -> Vec<u8> = encode;
/// ```
pub struct WorthQuerySameCommitName<K> {
    reference: SameCommitNameReference,
    _kind: PhantomData<fn() -> K>,
}

impl<K> WorthQuerySameCommitName<K> {
    pub(crate) fn issued(reference: SameCommitNameReference) -> Self {
        Self {
            reference,
            _kind: PhantomData,
        }
    }

    pub(super) fn reference(&self) -> &SameCommitNameReference {
        &self.reference
    }
}

impl<K> Clone for WorthQuerySameCommitName<K> {
    fn clone(&self) -> Self {
        Self::issued(self.reference.clone())
    }
}

impl<K> PartialEq for WorthQuerySameCommitName<K> {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
    }
}

impl<K> Eq for WorthQuerySameCommitName<K> {}

impl<K> fmt::Debug for WorthQuerySameCommitName<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthQuerySameCommitName")
            .field("kind", &std::any::type_name::<K>())
            .field("output", &self.reference.output)
            .finish()
    }
}

/// Kind-erased same-commit reference: the candidate that issued it and the
/// ordinal of the named output it designates there.
#[derive(Clone)]
pub struct SameCommitNameReference {
    candidate: Arc<()>,
    output: usize,
}

impl SameCommitNameReference {
    pub(crate) fn issued(candidate: Arc<()>, output: usize) -> Self {
        Self { candidate, output }
    }

    /// Whether `candidate` is the one that issued this reference.
    pub fn issued_by(&self, candidate: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.candidate, candidate)
    }

    /// Ordinal of the designated named output in its candidate.
    pub const fn output(&self) -> usize {
        self.output
    }
}

impl PartialEq for SameCommitNameReference {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.candidate, &other.candidate) && self.output == other.output
    }
}

impl Eq for SameCommitNameReference {}

impl fmt::Debug for SameCommitNameReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SameCommitNameReference")
            .field("output", &self.output)
            .finish_non_exhaustive()
    }
}

/// Where one same-commit reference sits in a canonical discriminator: the
/// byte offset of its [`SAME_COMMIT_NAME_PLACEHOLDER`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SameCommitNameSlot {
    offset: usize,
    reference: SameCommitNameReference,
}

impl SameCommitNameSlot {
    pub(super) fn new(offset: usize, reference: SameCommitNameReference) -> Self {
        Self { offset, reference }
    }

    pub(super) fn rebased(self, base: usize) -> Self {
        Self {
            offset: base + self.offset,
            reference: self.reference,
        }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub fn reference(&self) -> &SameCommitNameReference {
        &self.reference
    }
}
