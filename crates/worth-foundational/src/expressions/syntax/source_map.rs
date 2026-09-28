//! Source provenance. Spans locate authored text; they never carry meaning.

/// A half-open UTF-8 byte range in authored expression source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceSpan {
    start: u32,
    end: u32,
}

impl SourceSpan {
    pub(crate) const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn start(self) -> u32 {
        self.start
    }

    pub const fn end(self) -> u32 {
        self.end
    }

    pub(crate) const fn to(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}

/// Where one canonical program node came from.
///
/// An installed function body keeps its own map; a call node maps to the
/// authored call, and occurrences inside the body map through the function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceOrigin {
    span: Option<SourceSpan>,
    syntax_node: u32,
}

impl SourceOrigin {
    pub(crate) const fn new(span: Option<SourceSpan>, syntax_node: u32) -> Self {
        Self { span, syntax_node }
    }

    /// The authored source clause, when the draft came from source text.
    pub const fn span(self) -> Option<SourceSpan> {
        self.span
    }

    /// The syntax node ordinal in the admitted draft.
    pub const fn syntax_node(self) -> u32 {
        self.syntax_node
    }
}

/// Maps each canonical program node occurrence to its authored origin.
///
/// Bound beside canonical meaning, not inside it: equivalent source with
/// different spacing shares one program identity but keeps its own map.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionSourceMap {
    origins: Vec<SourceOrigin>,
}

impl ExpressionSourceMap {
    pub(crate) fn new(origins: Vec<SourceOrigin>) -> Self {
        Self { origins }
    }

    /// The origin of canonical node `index`, in canonical node order.
    pub fn origin(&self, index: usize) -> Option<SourceOrigin> {
        self.origins.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.origins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.origins.is_empty()
    }
}
