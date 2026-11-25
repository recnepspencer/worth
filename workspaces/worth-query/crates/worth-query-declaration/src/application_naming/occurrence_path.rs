use std::fmt;
use std::str::FromStr;

/// Separates the segments of a rendered occurrence key.
const SEGMENT_SEPARATOR: char = '/';
/// Opens and closes a segment's index in a rendered occurrence key.
const INDEX_OPEN: char = '[';
const INDEX_CLOSE: char = ']';
/// Opens and closes a segment's anchor path in a rendered occurrence key.
const ANCHOR_OPEN: char = '(';
const ANCHOR_CLOSE: char = ')';

/// The authored occurrence a name is born in, as a path of segments rather
/// than one packed string: a feature, then the pattern instance inside it,
/// then the hole inside that.
///
/// Every segment is a name and, for a repeated occurrence, a numeric index
/// (`instance[2]`), or, for an occurrence placed relative to another one, an
/// anchor path (`placement(base/instance[2])`). Paths order by segment; a
/// plain segment sorts before the indexed ones of the same name, indexes order
/// by number (`instance[10]` sorts after `instance[9]`), and anchored
/// segments sort last, by anchor path.
///
/// A path renders to one canonical occurrence key, and no two paths render to
/// the same key: a segment name can hold neither whitespace nor the
/// characters `/`, `[`, `]`, `(` and `)` the rendering uses.
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
///
/// let placed = OccurrencePath::root("gen")?.anchored("placement", &hole(2)?)?;
/// assert_eq!(placed.to_occurrence_key(), "gen/placement(pattern_1/instance[2]/hole)");
/// assert_eq!(placed.segments()[1].anchor(), Some(&hole(2)?));
/// assert_eq!(placed.to_occurrence_key().parse::<OccurrencePath>()?, placed);
/// # Ok::<(), OccurrencePathDenial>(())
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrencePath {
    segments: Vec<OccurrenceSegment>,
}

/// One step of an [`OccurrencePath`]: a name and, for a repeated occurrence,
/// which repetition it is, or, for an anchored occurrence, the path it is
/// anchored to.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrenceSegment {
    name: String,
    form: SegmentForm,
}

/// The three unambiguous segment forms, in their canonical order.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum SegmentForm {
    Plain,
    Indexed(u32),
    Anchored(OccurrencePath),
}

impl OccurrenceSegment {
    fn new(name: &str, form: SegmentForm) -> Result<Self, OccurrencePathDenial> {
        if name.is_empty() {
            return Err(OccurrencePathDenial::EmptySegment);
        }
        let ambiguous = |ch: char| {
            ch.is_whitespace()
                || ch.is_control()
                || matches!(
                    ch,
                    SEGMENT_SEPARATOR | INDEX_OPEN | INDEX_CLOSE | ANCHOR_OPEN | ANCHOR_CLOSE
                )
        };
        if name.chars().any(ambiguous) {
            return Err(OccurrencePathDenial::InvalidSegment {
                segment: name.to_owned(),
            });
        }
        Ok(Self {
            name: name.to_owned(),
            form,
        })
    }

    /// Parses one rendered segment: `name`, `name[index]` or
    /// `name(anchor)`. The segment's first delimiter decides its form, since
    /// a name holds none.
    fn parse(rendered: &str) -> Result<Self, OccurrencePathDenial> {
        let Some(open) = rendered.find([INDEX_OPEN, ANCHOR_OPEN]) else {
            return Self::new(rendered, SegmentForm::Plain);
        };
        if rendered[open..].starts_with(ANCHOR_OPEN) {
            return Self::parse_anchored(rendered, open);
        }
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
        Self::new(&rendered[..open], SegmentForm::Indexed(index))
    }

    /// Parses `name(anchor)`: the parenthesis opened at `open` must close
    /// exactly at the segment's end, around a non-empty canonical path.
    ///
    /// Complexity: O(len · depth) for the segment's length and anchor
    /// nesting depth.
    fn parse_anchored(rendered: &str, open: usize) -> Result<Self, OccurrencePathDenial> {
        let invalid_anchor = || OccurrencePathDenial::InvalidAnchor {
            segment: rendered.to_owned(),
        };
        let inner = rendered[open + ANCHOR_OPEN.len_utf8()..]
            .strip_suffix(ANCHOR_CLOSE)
            .ok_or_else(invalid_anchor)?;
        if inner.is_empty() || !balanced(inner) {
            return Err(invalid_anchor());
        }
        let anchor = OccurrencePath::parse(inner)?;
        Self::new(&rendered[..open], SegmentForm::Anchored(anchor))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Which repetition this segment is, compared numerically.
    pub const fn index(&self) -> Option<u32> {
        match self.form {
            SegmentForm::Indexed(index) => Some(index),
            SegmentForm::Plain | SegmentForm::Anchored(_) => None,
        }
    }

    /// The occurrence this segment is anchored to.
    pub const fn anchor(&self) -> Option<&OccurrencePath> {
        match &self.form {
            SegmentForm::Anchored(anchor) => Some(anchor),
            SegmentForm::Plain | SegmentForm::Indexed(_) => None,
        }
    }
}

/// Whether every anchor parenthesis in `text` closes, and none closes before
/// it opens.
fn balanced(text: &str) -> bool {
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            ANCHOR_OPEN => depth += 1,
            ANCHOR_CLOSE => match depth.checked_sub(1) {
                Some(outer) => depth = outer,
                None => return false,
            },
            _ => {}
        }
    }
    depth == 0
}

/// Splits a rendered key at the separators outside every anchor.
fn top_level_segments(key: &str) -> Vec<&str> {
    let (mut segments, mut depth, mut start) = (Vec::new(), 0usize, 0);
    for (position, ch) in key.char_indices() {
        match ch {
            ANCHOR_OPEN => depth += 1,
            ANCHOR_CLOSE => depth = depth.saturating_sub(1),
            SEGMENT_SEPARATOR if depth == 0 => {
                segments.push(&key[start..position]);
                start = position + SEGMENT_SEPARATOR.len_utf8();
            }
            _ => {}
        }
    }
    segments.push(&key[start..]);
    segments
}

impl fmt::Display for OccurrenceSegment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)?;
        match &self.form {
            SegmentForm::Plain => Ok(()),
            SegmentForm::Indexed(index) => write!(formatter, "{INDEX_OPEN}{index}{INDEX_CLOSE}"),
            SegmentForm::Anchored(anchor) => {
                write!(formatter, "{ANCHOR_OPEN}{anchor}{ANCHOR_CLOSE}")
            }
        }
    }
}

impl OccurrencePath {
    /// Starts a path at an authored identity, such as a feature's own key.
    pub fn root(key: &str) -> Result<Self, OccurrencePathDenial> {
        Ok(Self {
            segments: vec![OccurrenceSegment::new(key, SegmentForm::Plain)?],
        })
    }

    /// Descends into the single occurrence named `segment`.
    pub fn child(mut self, segment: &str) -> Result<Self, OccurrencePathDenial> {
        self.segments
            .push(OccurrenceSegment::new(segment, SegmentForm::Plain)?);
        Ok(self)
    }

    /// Descends into repetition `index` of the occurrence named `name`.
    pub fn index(mut self, name: &str, index: u32) -> Result<Self, OccurrencePathDenial> {
        self.segments
            .push(OccurrenceSegment::new(name, SegmentForm::Indexed(index))?);
        Ok(self)
    }

    /// Descends into the occurrence named `name` that is anchored to another
    /// occurrence, such as a generated placement of the feature at `anchor`.
    pub fn anchored(
        mut self,
        name: &str,
        anchor: &OccurrencePath,
    ) -> Result<Self, OccurrencePathDenial> {
        let form = SegmentForm::Anchored(anchor.clone());
        self.segments.push(OccurrenceSegment::new(name, form)?);
        Ok(self)
    }

    /// Reads back a key [`Self::to_occurrence_key`] rendered. Refuses any
    /// other spelling of the same path, so one path has one key.
    ///
    /// Complexity: O(len · depth) for the key's length and anchor nesting
    /// depth.
    pub fn parse(occurrence_key: &str) -> Result<Self, OccurrencePathDenial> {
        let segments = top_level_segments(occurrence_key)
            .into_iter()
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
    /// rendering's own delimiters (`/`, `[`, `]`, `(`, `)`).
    InvalidSegment { segment: String },
    /// A rendered segment's index is not the canonical decimal form of a
    /// `u32` closing the segment.
    InvalidIndex { segment: String },
    /// A rendered segment's anchor is empty, unbalanced or does not close the
    /// segment.
    InvalidAnchor { segment: String },
}

impl fmt::Display for OccurrencePathDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySegment => formatter.write_str("occurrence path segment is empty"),
            Self::InvalidSegment { segment } => write!(
                formatter,
                "occurrence path segment `{segment}` holds whitespace or one of `/`, `[`, `]`, `(`, `)`"
            ),
            Self::InvalidIndex { segment } => write!(
                formatter,
                "occurrence path segment `{segment}` does not end in a canonical `[index]`"
            ),
            Self::InvalidAnchor { segment } => write!(
                formatter,
                "occurrence path segment `{segment}` does not end in one non-empty `(anchor)`"
            ),
        }
    }
}

impl std::error::Error for OccurrencePathDenial {}
