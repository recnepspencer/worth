use std::cmp::Ordering;
use std::fmt;

use super::{NameSelection, WorthQueryName};

/// A held reference: an admitted name of kind `K` and how it chooses among
/// that name's live descendants when it is resolved.
///
/// It is built only from a name WORTH admitted, so it is no more forgeable
/// than the name it holds. Like the name, a target of a
/// [`super::WorthQueryNamedKind`] is a canonical value a record column or an
/// action input holds, with the selection stored beside the name.
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     NameSelection, NamedTarget, WorthQueryName,
/// };
///
/// struct Edge;
///
/// fn hold(edge: WorthQueryName<Edge>) {
///     let exact = NamedTarget::exact(edge);
///     assert_eq!(exact, NamedTarget::new(edge, NameSelection::Exact));
///     assert_eq!(exact.selection(), &NameSelection::Exact);
///     let (name, selection) = NamedTarget::all_descendants(edge).into_parts();
///     assert_eq!((name, selection), (edge, NameSelection::AllDescendants));
/// }
///
/// let _check: fn(WorthQueryName<Edge>) = hold;
/// ```
///
/// Raw bytes cannot become a target:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::NamedTarget;
///
/// struct Edge;
///
/// let _forged: NamedTarget<Edge> = NamedTarget::exact([0u8; 32]);
/// ```
///
/// Nor can a struct literal (the fields are private):
///
/// ```compile_fail,E0451
/// use worth_query_declaration::facade::application_naming::{
///     NameSelection, NamedTarget, WorthQueryName,
/// };
///
/// struct Edge;
///
/// fn forge(name: WorthQueryName<Edge>) -> NamedTarget<Edge> {
///     NamedTarget { name, selection: NameSelection::Exact }
/// }
/// ```
///
/// A target of one kind is not a target of another:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::NamedTarget;
///
/// struct Face;
/// struct Edge;
///
/// fn require_face(_target: NamedTarget<Face>) {}
///
/// fn misuse(edge: NamedTarget<Edge>) {
///     require_face(edge);
/// }
/// ```
pub struct NamedTarget<K> {
    name: WorthQueryName<K>,
    selection: NameSelection,
}

impl<K> NamedTarget<K> {
    /// Holds `name` with an explicit selection.
    pub const fn new(name: WorthQueryName<K>, selection: NameSelection) -> Self {
        Self { name, selection }
    }

    /// Holds `name` expecting exactly one live descendant.
    pub const fn exact(name: WorthQueryName<K>) -> Self {
        Self::new(name, NameSelection::Exact)
    }

    /// Holds `name` standing for every live descendant.
    pub const fn all_descendants(name: WorthQueryName<K>) -> Self {
        Self::new(name, NameSelection::AllDescendants)
    }

    pub const fn name(&self) -> WorthQueryName<K> {
        self.name
    }

    pub const fn selection(&self) -> &NameSelection {
        &self.selection
    }

    pub fn into_parts(self) -> (WorthQueryName<K>, NameSelection) {
        (self.name, self.selection)
    }
}

impl<K> Clone for NamedTarget<K> {
    fn clone(&self) -> Self {
        Self::new(self.name, self.selection.clone())
    }
}

impl<K> PartialEq for NamedTarget<K> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.selection == other.selection
    }
}

impl<K> Eq for NamedTarget<K> {}

impl<K> PartialOrd for NamedTarget<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// By name, then by selection.
impl<K> Ord for NamedTarget<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.selection.cmp(&other.selection))
    }
}

impl<K> fmt::Debug for NamedTarget<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NamedTarget")
            .field("name", &self.name)
            .field("selection", &self.selection)
            .finish()
    }
}
