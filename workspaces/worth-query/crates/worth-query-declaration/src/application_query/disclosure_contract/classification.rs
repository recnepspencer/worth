/// A typed disclosure classification: the named sensitivity class a governed
/// or installed-policy disclosure contract carries.
///
/// Classifications are declared types, never strings at the contract site.
/// Declare one with [`crate::worth_query_disclosure_classification!`]; its
/// classification name is the declared type name.
///
/// ```
/// use worth_query_declaration::facade::application_query::{
///     ApplicationDisclosureClassification, ApplicationQueryDisclosureContract,
/// };
/// worth_query_declaration::worth_query_disclosure_classification!(pub ProjectReadDisclosure);
///
/// assert_eq!(ProjectReadDisclosure::CLASSIFICATION, "ProjectReadDisclosure");
/// let contract = ApplicationQueryDisclosureContract::installed_policy(ProjectReadDisclosure);
/// assert_eq!(contract.classification(), "ProjectReadDisclosure");
/// ```
///
/// A raw classification string does not compile:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
/// let _ = ApplicationQueryDisclosureContract::installed_policy("restricted");
/// ```
pub trait ApplicationDisclosureClassification: Copy + 'static {
    /// The declared classification name: the classification type's name.
    const CLASSIFICATION: &'static str;
}
