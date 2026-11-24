use std::fmt;
use std::str::FromStr;

/// Separates the segments of a rendered occurrence key.
const SEGMENT_SEPARATOR: char = '/';
/// Opens and closes a segment's index in a rendered occurrence key.
const INDEX_OPEN: char = '[';
const INDEX_CLOSE: char = ']';

/// The authored occurrence a name is born in, as a path of segments rather
/// than one packed string: a feature, then the pattern instance inside it,
/// then the hole inside that.
///
/// Every segment is a name and, for a repeated occurrence, a numeric index.
/// Paths order by segment, and indexes order by number: `instance[10]` sorts
/// after `instance[9]`.
///
/// A path renders to one canonical occurrence key, and no two paths render to
/// the same key: a segment name can hold neither whitespace nor the
/// characters `/`, `[` and `]` the rendering uses.
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     OccurrencePath, OccurrencePathDenial,
/// };
///
/// let hole = |instance: u32| -> Result<OccurrencePath, OccurrencePathDenial> {
///     OccurrencePath::root("pattern_1")?.index("instance", instance)?.child("hole")
/// };
/// assert_eq!(hole(2)?.to_occurrence_key(), "pattern_1/instance[2]/hole");
/// assert!(hole(10)? > hole(9)? && hole(9)? > hole(2)?);
/// assert_eq!("pattern_1/instance[2]/hole".parse::<OccurrencePath>()?, hole(2)?);
/// assert_eq!(
///     OccurrencePath::root("a/b"),
///     Err(OccurrencePathDenial::InvalidSegment { segment: "a/b".into() })
/// );
/// # Ok::<(), OccurrencePathDenial>(())
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrencePath {
    segments: Vec<OccurrenceSegment>,
}

/// One step of an [`OccurrencePath`]: a name and, for a repeated occurrence,
/// which repetition it is.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrenceSegment {
    name: String,
    index: Option<u32>,
}

impl OccurrenceSegment {
    fn new(name: &str, index: Option<u32>) -> Result<Self, OccurrencePathDenial> {
        if name.is_empty() {
            return Err(OccurrencePathDenial::EmptySegment);
        }
        let ambiguous = |ch: char| {
            ch.is_whitespace()
                || ch.is_control()
                || matches!(ch, SEGMENT_SEPARATOR | INDEX_OPEN | INDEX_CLOSE)
        };
        if name.chars().any(ambiguous) {
            return Err(OccurrencePathDenial::InvalidSegment {
                segment: name.to_owned(),
            });
        }
        Ok(Self {
            name: name.to_owned(),
            index,
        })
    }

    /// Parses one rendered segment: `name` or `name[index]`.
    fn parse(rendered: &str) -> Result<Self, OccurrencePathDenial> {
        let Some(open) = rendered.find(INDEX_OPEN) else {
            return Self::new(rendered, None);
        };
        let invalid_index = || OccurrencePathDenial::InvalidIndex {
            segment: rendered.to_owned(),
        };
        let digits = rendered[open + INDEX_OPEN.len_utf8()..]
            .strip_suffix(INDEX_CLOSE)
            .ok_or_else(invalid_index)?;
        // Only the rendering `index` writes parses back: plain decimal digits
        // with no sign and no leading zero.
        let canonical = digits.bytes().all(|byte| byte.is_ascii_digit())
            && (digits == "0" || !digits.starts_with('0'));
        let index = digits
            .parse::<u32>()
            .ok()
            .filter(|_| canonical)
            .ok_or_else(invalid_index)?;
        Self::new(&rendered[..open], Some(index))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Which repetition this segment is, compared numerically.
    pub const fn index(&self) -> Option<u32> {
        self.index
    }
}

impl fmt::Display for OccurrenceSegment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)?;
        match self.index {
            Some(index) => write!(formatter, "{INDEX_OPEN}{index}{INDEX_CLOSE}"),
            None => Ok(()),
        }
    }
}

impl OccurrencePath {
    /// Starts a path at an authored identity, such as a feature's own key.
    pub fn root(key: &str) -> Result<Self, OccurrencePathDenial> {
        Ok(Self {
            segments: vec![OccurrenceSegment::new(key, None)?],
        })
    }

    /// Descends into the single occurrence named `segment`.
    pub fn child(mut self, segment: &str) -> Result<Self, OccurrencePathDenial> {
        self.segments.push(OccurrenceSegment::new(segment, None)?);
        Ok(self)
    }

    /// Descends into repetition `index` of the occurrence named `name`.
    pub fn index(mut self, name: &str, index: u32) -> Result<Self, OccurrencePathDenial> {
        self.segments
            .push(OccurrenceSegment::new(name, Some(index))?);
        Ok(self)
    }

    /// Reads back a key [`Self::to_occurrence_key`] rendered. Refuses any
    /// other spelling of the same path, so one path has one key.
    ///
    /// Complexity: O(len) for the key's length.
    pub fn parse(occurrence_key: &str) -> Result<Self, OccurrencePathDenial> {
        let segments = occurrence_key
            .split(SEGMENT_SEPARATOR)
            .map(OccurrenceSegment::parse)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { segments })
    }

    /// The segments from the root down; never empty.
    pub fn segments(&self) -> &[OccurrenceSegment] {
        &self.segments
    }

    /// The canonical occurrence key a birth under this path is minted from.
    ///
    /// Complexity: O(len) for the rendered length.
    pub fn to_occurrence_key(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for OccurrencePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, segment) in self.segments.iter().enumerate() {
            if position > 0 {
                write!(formatter, "{SEGMENT_SEPARATOR}")?;
            }
            write!(formatter, "{segment}")?;
        }
        Ok(())
    }
}

impl FromStr for OccurrencePath {
    type Err = OccurrencePathDenial;

    fn from_str(occurrence_key: &str) -> Result<Self, Self::Err> {
        Self::parse(occurrence_key)
    }
}

impl From<&OccurrencePath> for String {
    fn from(path: &OccurrencePath) -> Self {
        path.to_occurrence_key()
    }
}

impl From<OccurrencePath> for String {
    fn from(path: OccurrencePath) -> Self {
        path.to_occurrence_key()
    }
}

/// Typed refusal of a segment or rendered key an [`OccurrencePath`] cannot
/// hold without becoming ambiguous.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OccurrencePathDenial {
    /// A segment has no name.
    EmptySegment,
    /// A segment name holds whitespace, a control character or one of the
    /// rendering's own delimiters (`/`, `[`, `]`).
    InvalidSegment { segment: String },
    /// A rendered segment's index is not the canonical decimal form of a
    /// `u32` closing the segment.
    InvalidIndex { segment: String },
}

impl fmt::Display for OccurrencePathDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySegment => formatter.write_str("occurrence path segment is empty"),
            Self::InvalidSegment { segment } => write!(
                formatter,
                "occurrence path segment `{segment}` holds whitespace or one of `/`, `[`, `]`"
            ),
            Self::InvalidIndex { segment } => write!(
                formatter,
                "occurrence path segment `{segment}` does not end in a canonical `[index]`"
            ),
        }
    }
}

impl std::error::Error for OccurrencePathDenial {}
