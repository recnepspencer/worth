//! A recorded name's birth as a reader admitted it, and its typed decode.

use std::fmt;

use super::{
    AdmittedNameReferences, ApplicationNameDiscriminator, ApplicationNameDiscriminatorDecode,
    ApplicationNameDiscriminatorDecodeDenial, ApplicationNameDiscriminatorDecoder, OccurrencePath,
    OccurrencePathDenial, WorthQueryName, WorthQueryNameRole, NAME_ROLE_INDEX_IDENTITY,
};

/// The birth of a name recorded and visible at a tracked read's basis:
/// the occurrence, role path, parents and canonical discriminator it was
/// minted from.
///
/// A reader's `birth_of` returns one. Its parents and every name its
/// discriminator references were admitted by that read at the same basis;
/// decoding admits no other name.
///
/// Decode the role and discriminator as the types the scheme declared:
/// `decode_role::<R>()`, `decode_discriminator::<D>()`, or both at once with
/// [`Self::decode`]. A birth under an indexed role (`Instance(3)`) decodes
/// its index into the role; the discriminator is whatever follows it.
pub struct RecordedNameBirth<K> {
    name: WorthQueryName<K>,
    occurrence_key: String,
    role_path: String,
    parents: Vec<WorthQueryName<K>>,
    canonical_discriminator: Vec<u8>,
    admitted_references: AdmittedNameReferences,
}

/// A recorded birth decoded as its scheme's typed role `R` and
/// discriminator `D`.
pub struct NameBirth<K, R, D> {
    pub name: WorthQueryName<K>,
    pub occurrence: OccurrencePath,
    pub role: R,
    /// The birth's parent names, sorted by digest as they were minted.
    pub parents: Vec<WorthQueryName<K>>,
    /// `None` for a birth without a typed discriminator: no discriminator at
    /// all, or only its role index.
    pub discriminator: Option<D>,
}

/// Why a recorded birth does not decode as the requested types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameBirthDecodeDenial {
    /// The occurrence key is not a canonical [`OccurrencePath`].
    Occurrence(OccurrencePathDenial),
    /// The role type declares no role for this role path and index.
    UnknownRole {
        role_path: String,
        index: Option<u32>,
    },
    /// The canonical discriminator is not the requested type's encoding.
    Discriminator(ApplicationNameDiscriminatorDecodeDenial),
}

impl fmt::Display for NameBirthDecodeDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occurrence(denial) => write!(formatter, "birth occurrence: {denial}"),
            Self::UnknownRole { role_path, index } => {
                write!(
                    formatter,
                    "no declared role `{role_path}` with index {index:?}"
                )
            }
            Self::Discriminator(denial) => write!(formatter, "birth discriminator: {denial}"),
        }
    }
}

impl std::error::Error for NameBirthDecodeDenial {}

impl From<ApplicationNameDiscriminatorDecodeDenial> for NameBirthDecodeDenial {
    fn from(denial: ApplicationNameDiscriminatorDecodeDenial) -> Self {
        Self::Discriminator(denial)
    }
}

type Decoded<T> = Result<T, NameBirthDecodeDenial>;

impl<K> RecordedNameBirth<K> {
    /// Only the admission authority builds one, from a birth it read at a
    /// basis where `name`, every parent and every admitted reference are
    /// recorded and visible.
    pub(crate) fn from_admitted_parts(
        name: WorthQueryName<K>,
        occurrence_key: String,
        role_path: String,
        parents: Vec<WorthQueryName<K>>,
        canonical_discriminator: Vec<u8>,
        admitted_references: AdmittedNameReferences,
    ) -> Self {
        Self {
            name,
            occurrence_key,
            role_path,
            parents,
            canonical_discriminator,
            admitted_references,
        }
    }

    /// The name this birth minted.
    pub const fn name(&self) -> WorthQueryName<K> {
        self.name
    }

    /// The canonical occurrence key the name was minted from.
    pub fn occurrence_key(&self) -> &str {
        &self.occurrence_key
    }

    /// The occurrence as a path; refuses a key no [`OccurrencePath`] renders
    /// (a birth named from a bare application key).
    pub fn occurrence(&self) -> Result<OccurrencePath, OccurrencePathDenial> {
        OccurrencePath::parse(&self.occurrence_key)
    }

    /// The declared role the name was born under.
    pub fn role_path(&self) -> &str {
        &self.role_path
    }

    /// The parent names, sorted by digest as they were minted: the split
    /// parent of a piece, the parents of a minted merge result, none for a
    /// plain birth.
    pub fn parents(&self) -> &[WorthQueryName<K>] {
        &self.parents
    }

    /// The canonical discriminator bytes; empty when the birth has none.
    pub fn canonical_discriminator(&self) -> &[u8] {
        &self.canonical_discriminator
    }

    /// Which repetition of an indexed role the birth is.
    pub fn role_index(&self) -> Decoded<Option<u32>> {
        if self.canonical_discriminator.is_empty() {
            return Ok(None);
        }
        let mut decoder = self.decoder();
        // An index-only discriminator must carry its index.
        if decoder.header()? == NAME_ROLE_INDEX_IDENTITY {
            return Ok(Some(decoder.role_index()?));
        }
        Ok(decoder.leading_role_index()?)
    }

    /// The role the birth was born under, as the scheme's typed role.
    pub fn decode_role<R: WorthQueryNameRole>(&self) -> Decoded<R> {
        let index = self.role_index()?;
        R::from_role_parts(&self.role_path, index).ok_or_else(|| {
            NameBirthDecodeDenial::UnknownRole {
                role_path: self.role_path.clone(),
                index,
            }
        })
    }

    /// The birth's typed discriminator, after its role index; `None` when
    /// it has none. Refuses bytes of another discriminator type, malformed or
    /// trailing bytes, and any name reference this read did not admit.
    pub fn decode_discriminator<D: ApplicationNameDiscriminatorDecode>(
        &self,
    ) -> Decoded<Option<D>> {
        if self.canonical_discriminator.is_empty() {
            return Ok(None);
        }
        let mut decoder = self.decoder();
        let identity = decoder.header()?;
        let requested = <D as ApplicationNameDiscriminator>::DISCRIMINATOR_IDENTITY;
        let mismatch = || ApplicationNameDiscriminatorDecodeDenial::IdentityMismatch {
            expected: requested.to_owned(),
            found: identity.to_owned(),
        };
        // A birth told apart by its role index alone carries exactly that
        // index and no typed discriminator: nothing for a plain `D`, and a
        // mismatch for a `D` that itself expects a discriminator after it.
        if identity == NAME_ROLE_INDEX_IDENTITY && requested != NAME_ROLE_INDEX_IDENTITY {
            if D::LEADS_WITH_ROLE_INDEX {
                return Err(mismatch().into());
            }
            decoder.role_index()?;
            decoder.finish()?;
            return Ok(None);
        }
        if identity != requested {
            return Err(mismatch().into());
        }
        if identity == NAME_ROLE_INDEX_IDENTITY && !D::LEADS_WITH_ROLE_INDEX {
            decoder.role_index()?;
        } else if !D::LEADS_WITH_ROLE_INDEX {
            decoder.leading_role_index()?;
        }
        let decoded = D::decode_canonical(&mut decoder)?;
        decoder.finish()?;
        Ok(Some(decoded))
    }

    /// The whole birth as its scheme's typed role and discriminator.
    pub fn decode<R, D>(&self) -> Decoded<NameBirth<K, R, D>>
    where
        R: WorthQueryNameRole,
        D: ApplicationNameDiscriminatorDecode,
    {
        Ok(NameBirth {
            name: self.name,
            occurrence: self
                .occurrence()
                .map_err(NameBirthDecodeDenial::Occurrence)?,
            role: self.decode_role()?,
            parents: self.parents.clone(),
            discriminator: self.decode_discriminator()?,
        })
    }

    fn decoder(&self) -> ApplicationNameDiscriminatorDecoder<'_> {
        ApplicationNameDiscriminatorDecoder::new(
            &self.canonical_discriminator,
            &self.admitted_references,
        )
    }
}

impl<K> Clone for RecordedNameBirth<K> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            occurrence_key: self.occurrence_key.clone(),
            role_path: self.role_path.clone(),
            parents: self.parents.clone(),
            canonical_discriminator: self.canonical_discriminator.clone(),
            admitted_references: self.admitted_references.clone(),
        }
    }
}

/// Two reads of one recorded birth are equal; what each admitted is part of
/// what it can decode.
impl<K> PartialEq for RecordedNameBirth<K> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.occurrence_key == other.occurrence_key
            && self.role_path == other.role_path
            && self.parents == other.parents
            && self.canonical_discriminator == other.canonical_discriminator
            && self.admitted_references == other.admitted_references
    }
}

impl<K> Eq for RecordedNameBirth<K> {}

impl<K> fmt::Debug for RecordedNameBirth<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecordedNameBirth")
            .field("name", &self.name)
            .field("occurrence_key", &self.occurrence_key)
            .field("role_path", &self.role_path)
            .field("parents", &self.parents)
            .field(
                "canonical_discriminator_len",
                &self.canonical_discriminator.len(),
            )
            .finish()
    }
}

impl<K, R: Clone, D: Clone> Clone for NameBirth<K, R, D> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            occurrence: self.occurrence.clone(),
            role: self.role.clone(),
            parents: self.parents.clone(),
            discriminator: self.discriminator.clone(),
        }
    }
}

impl<K, R: PartialEq, D: PartialEq> PartialEq for NameBirth<K, R, D> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.occurrence == other.occurrence
            && self.role == other.role
            && self.parents == other.parents
            && self.discriminator == other.discriminator
    }
}

impl<K, R: Eq, D: Eq> Eq for NameBirth<K, R, D> {}

impl<K, R: fmt::Debug, D: fmt::Debug> fmt::Debug for NameBirth<K, R, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NameBirth")
            .field("name", &self.name)
            .field("occurrence", &self.occurrence)
            .field("role", &self.role)
            .field("parents", &self.parents)
            .field("discriminator", &self.discriminator)
            .finish()
    }
}
