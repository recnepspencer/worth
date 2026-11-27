//! A rendered refusal is one safe sentence: never the typed refusal's
//! `Debug` form, a type name or a path.
use std::fmt::Debug;

use super::summary::{WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal};

/// `refusal` renders `category` in a sentence free of its own `Debug` text.
pub(super) fn assert_renders(
    refusal: &(impl WorthQueryUserFacingRefusal + Debug),
    category: Category,
) {
    let debug = format!("{refusal:?}");
    assert_eq!(refusal.refusal_category(), category, "{debug}");
    let message = refusal.user_message();
    assert!(!message.is_empty(), "{debug}");
    assert!(
        !message.contains("WorthQuery") && !message.contains("::") && !message.contains('{'),
        "{debug} rendered {message:?}"
    );
    let variant = debug.split(['(', ' ', '{']).next().unwrap_or_default();
    assert!(!message.contains(variant), "{debug} rendered {message:?}");
}
