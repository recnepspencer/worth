/// Declares a disclosure classification: a unit struct naming the sensitivity
/// class a governed or installed-policy disclosure contract carries.
///
/// Input: `vis Classification`. It generates `#[derive(Clone, Copy, Debug, Eq,
/// PartialEq)] vis struct Classification;` implementing
/// `ApplicationDisclosureClassification`, whose classification name is the type
/// name. Pass the value to `ApplicationQueryDisclosureContract::governed_by` or
/// `::installed_policy`; no classification string is ever written.
#[macro_export]
macro_rules! worth_query_disclosure_classification {
    ($(#[$meta:meta])* $vis:vis $Classification:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        $vis struct $Classification;

        impl $crate::facade::application_query::ApplicationDisclosureClassification
            for $Classification
        {
            const CLASSIFICATION: &'static str = stringify!($Classification);
        }
    };
}
