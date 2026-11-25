//! The canonical stored text of a held persistent name and held target.
//!
//! A stored name is `wqn1.<e|r>.<subject>.<digest>` and a stored target is
//! `wqt1.<e|r>.<subject>.<digest>.<selection>`: a versioned family tag, the
//! subject class and declared subject name of the kind it was stored as, the
//! 64-digit lowercase digest, and for a target one of `exact`, `all`,
//! `carriers.<digest>...` (ascending, at least one) or `side.<bytes>`.
//!
//! Decoding accepts exactly the text encoding writes for the same kind.
//! Another version, subject class or subject, a malformed digest and a
//! non-canonical selection each refuse typed; nothing is reinterpreted. A
//! decoded value is a held reference, not authority: every reader, selection
//! and discriminator admits it only against a recorded birth of that kind.

use super::kind::{subject_kind_of, subject_name_of};
use super::{
    ApplicationNameSubjectKind, ErasedWorthQueryName, NameCarrierSet, NameSelection,
    NameSideDiscriminator, NamedTarget, WorthQueryName, WorthQueryNamedKind,
};

const NAME_FAMILY: &str = "wqn";
const TARGET_FAMILY: &str = "wqt";
/// The stored-name grammar version this build writes and reads.
pub(crate) const VERSION: u32 = 1;

/// Why a stored persistent name or target was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoredNameDenial {
    /// The text does not follow the stored-name grammar.
    Malformed,
    /// The text is a stored name of a version this build does not read.
    UnsupportedVersion { observed: u32 },
    /// The text was stored as another subject class (entity or relation).
    SubjectKindMismatch,
    /// The text was stored as a name of another declared subject.
    SubjectMismatch,
    /// The target's selection is not in canonical form.
    NonCanonicalSelection,
    /// The kind's declared subject name cannot be written in stored text.
    UnencodableSubject,
}

impl StoredNameDenial {
    /// A stable description, also the reason a value binding refuses with.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Malformed => "stored persistent name is malformed",
            Self::UnsupportedVersion { .. } => "stored persistent name has an unsupported version",
            Self::SubjectKindMismatch => "stored persistent name belongs to another subject class",
            Self::SubjectMismatch => "stored persistent name belongs to another kind",
            Self::NonCanonicalSelection => "stored name target selection is not canonical",
            Self::UnencodableSubject => "kind subject name cannot be stored",
        }
    }
}

impl std::fmt::Display for StoredNameDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.reason())
    }
}

impl std::error::Error for StoredNameDenial {}

/// The stored text of `name` as a name of kind `K`.
pub(crate) fn encode_name<K: WorthQueryNamedKind>(
    name: &WorthQueryName<K>,
) -> Result<String, StoredNameDenial> {
    let mut text = head::<K>(NAME_FAMILY)?;
    push_hex(&mut text, name.as_bytes());
    Ok(text)
}

/// The name stored in `text`, only when it was stored as kind `K`.
pub(crate) fn decode_name<K: WorthQueryNamedKind>(
    text: &str,
) -> Result<WorthQueryName<K>, StoredNameDenial> {
    let mut parts = text.split('.');
    let digest = parse_head::<K>(&mut parts, NAME_FAMILY)?;
    if parts.next().is_some() {
        return Err(StoredNameDenial::Malformed);
    }
    Ok(WorthQueryName::from_admitted_digest(digest))
}

/// The stored text of `target` as a target of kind `K`.
pub(crate) fn encode_target<K: WorthQueryNamedKind>(
    target: &NamedTarget<K>,
) -> Result<String, StoredNameDenial> {
    let mut text = head::<K>(TARGET_FAMILY)?;
    push_hex(&mut text, target.name().as_bytes());
    match target.selection() {
        NameSelection::Exact => text.push_str(".exact"),
        NameSelection::AllDescendants => text.push_str(".all"),
        NameSelection::BoundedByCarriers { carriers } => {
            text.push_str(".carriers");
            for carrier in carriers.as_slice() {
                text.push('.');
                push_hex(&mut text, carrier.as_bytes());
            }
        }
        NameSelection::KeepSide { discriminator } => {
            text.push_str(".side.");
            push_hex(&mut text, discriminator.as_bytes());
        }
    }
    Ok(text)
}

/// The target stored in `text`, only when it was stored as kind `K`.
pub(crate) fn decode_target<K: WorthQueryNamedKind>(
    text: &str,
) -> Result<NamedTarget<K>, StoredNameDenial> {
    let mut parts = text.split('.');
    let digest = parse_head::<K>(&mut parts, TARGET_FAMILY)?;
    let selection = match parts.next().ok_or(StoredNameDenial::Malformed)? {
        "exact" => NameSelection::Exact,
        "all" => NameSelection::AllDescendants,
        "carriers" => {
            let carriers = parts
                .by_ref()
                .map(|carrier| {
                    digest_of(carrier).map(ErasedWorthQueryName::from_admitted_name_bytes)
                })
                .collect::<Result<Vec<_>, _>>()?;
            NameSelection::BoundedByCarriers {
                carriers: NameCarrierSet::from_canonical(carriers)
                    .ok_or(StoredNameDenial::NonCanonicalSelection)?,
            }
        }
        "side" => {
            let bytes = bytes_of(parts.next().ok_or(StoredNameDenial::Malformed)?)?;
            NameSelection::KeepSide {
                discriminator: NameSideDiscriminator::from_canonical(bytes)
                    .ok_or(StoredNameDenial::NonCanonicalSelection)?,
            }
        }
        _ => return Err(StoredNameDenial::Malformed),
    };
    if parts.next().is_some() {
        return Err(StoredNameDenial::Malformed);
    }
    Ok(NamedTarget::new(
        WorthQueryName::from_admitted_digest(digest),
        selection,
    ))
}

/// `<family><version>.<class>.<subject>.`, refusing a subject name the
/// grammar cannot carry.
fn head<K: WorthQueryNamedKind>(family: &str) -> Result<String, StoredNameDenial> {
    let subject = subject_name_of::<K>();
    let class = class_tag(subject_kind_of::<K>()).ok_or(StoredNameDenial::UnencodableSubject)?;
    if subject.is_empty() || subject.contains('.') {
        return Err(StoredNameDenial::UnencodableSubject);
    }
    Ok(format!("{family}{VERSION}.{class}.{subject}."))
}

fn class_tag(kind: ApplicationNameSubjectKind) -> Option<&'static str> {
    match kind {
        ApplicationNameSubjectKind::Entity => Some("e"),
        ApplicationNameSubjectKind::Relation => Some("r"),
        ApplicationNameSubjectKind::Group => None,
    }
}

fn parse_head<'text, K: WorthQueryNamedKind>(
    parts: &mut impl Iterator<Item = &'text str>,
    family: &str,
) -> Result<[u8; 32], StoredNameDenial> {
    let mut next = || parts.next().ok_or(StoredNameDenial::Malformed);
    let version = next()?
        .strip_prefix(family)
        .ok_or(StoredNameDenial::Malformed)?;
    match version.parse::<u32>() {
        Ok(VERSION) if version == "1" => {}
        // Only a canonically spelled number names another version.
        Ok(observed) if observed.to_string() == version && observed != VERSION => {
            return Err(StoredNameDenial::UnsupportedVersion { observed });
        }
        _ => return Err(StoredNameDenial::Malformed),
    }
    let class = next()?;
    if !matches!(class, "e" | "r") {
        return Err(StoredNameDenial::Malformed);
    }
    if Some(class) != class_tag(subject_kind_of::<K>()) {
        return Err(StoredNameDenial::SubjectKindMismatch);
    }
    let subject = next()?;
    if subject.is_empty() {
        return Err(StoredNameDenial::Malformed);
    }
    if subject != subject_name_of::<K>() {
        return Err(StoredNameDenial::SubjectMismatch);
    }
    digest_of(next()?)
}

fn push_hex(text: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        text.push(HEX[(byte >> 4) as usize] as char);
        text.push(HEX[(byte & 0x0f) as usize] as char);
    }
}

fn digest_of(hex: &str) -> Result<[u8; 32], StoredNameDenial> {
    bytes_of(hex)?
        .try_into()
        .map_err(|_| StoredNameDenial::Malformed)
}

/// Lowercase hexadecimal only, so every byte string has one stored spelling.
fn bytes_of(hex: &str) -> Result<Vec<u8>, StoredNameDenial> {
    let nibble = |digit: u8| match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(StoredNameDenial::Malformed),
    };
    let digits = hex.as_bytes();
    if digits.is_empty() || digits.len() % 2 != 0 {
        return Err(StoredNameDenial::Malformed);
    }
    digits
        .chunks_exact(2)
        .map(|pair| Ok(nibble(pair[0])? << 4 | nibble(pair[1])?))
        .collect()
}
