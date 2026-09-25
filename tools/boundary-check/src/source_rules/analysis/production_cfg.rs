//! Whether source is compiled into a production build of its crate.
//!
//! A production build is not a test build, and its features are unknown, so a
//! `cfg` predicate is three-valued. Source is compiled out only when one of its
//! `cfg` predicates can never hold: `test`, `all(test, ..)`, `any()`, and
//! the like. Anything unparseable or feature-selected stays compiled, which
//! keeps rules that skip test code fail-closed.

use super::crate_modules::ModuleGraph;
use syn::punctuated::Punctuated;
use syn::{Attribute, Item, Meta, Token};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Holds {
    Never,
    Maybe,
    Always,
}

/// Whether these attributes remove their item from every production build.
pub(super) fn compiled_out_of_production(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<Meta>()
                .is_ok_and(|predicate| holds(&predicate) == Holds::Never)
    })
}

/// Whether a module, or any module declaring it, is compiled out of every
/// production build. A module declared more than once is compiled out only
/// when every declaration is.
pub(super) fn module_compiled_out_of_production(
    graph: &ModuleGraph,
    module_path: &[String],
) -> bool {
    (1..=module_path.len()).any(|depth| {
        let Some(parent) = graph.modules.get(&module_path[..depth - 1]) else {
            return false;
        };
        let name = &module_path[depth - 1];
        let mut declarations = parent
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(item_mod) if item_mod.ident == name => Some(item_mod),
                _ => None,
            })
            .peekable();
        declarations.peek().is_some()
            && declarations.all(|item_mod| compiled_out_of_production(&item_mod.attrs))
    })
}

fn holds(predicate: &Meta) -> Holds {
    match predicate {
        Meta::Path(path) if path.is_ident("test") => Holds::Never,
        Meta::List(list) if list.path.is_ident("not") => match list.parse_args::<Meta>() {
            Ok(inner) => match holds(&inner) {
                Holds::Never => Holds::Always,
                Holds::Maybe => Holds::Maybe,
                Holds::Always => Holds::Never,
            },
            Err(_) => Holds::Maybe,
        },
        Meta::List(list) if list.path.is_ident("all") || list.path.is_ident("any") => {
            let Ok(inner) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            else {
                return Holds::Maybe;
            };
            let values = inner.iter().map(holds);
            if list.path.is_ident("all") {
                values.min().unwrap_or(Holds::Always)
            } else {
                values.max().unwrap_or(Holds::Never)
            }
        }
        _ => Holds::Maybe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compiled_out(source: &str) -> bool {
        let item: Item = syn::parse_str(source).unwrap();
        let Item::Fn(function) = item else {
            panic!("fixture is a function")
        };
        compiled_out_of_production(&function.attrs)
    }

    #[test]
    fn only_predicates_that_can_never_hold_compile_source_out() {
        for source in [
            "#[cfg(test)] fn f() {}",
            "#[cfg(all(test, not(feature = \"certification-support\")))] fn f() {}",
            "#[cfg(any())] fn f() {}",
            "#[inline] #[cfg(all(unix, test))] fn f() {}",
        ] {
            assert!(compiled_out(source), "kept {source}");
        }
        for source in [
            "fn f() {}",
            "#[cfg(not(test))] fn f() {}",
            "#[cfg(not(any()))] fn f() {}",
            "#[cfg(feature = \"certification-support\")] fn f() {}",
            "#[cfg(any(test, feature = \"certification-support\"))] fn f() {}",
            "#[cfg(debug_assertions)] fn f() {}",
            "#[cfg_attr(test, allow(dead_code))] fn f() {}",
        ] {
            assert!(!compiled_out(source), "dropped {source}");
        }
    }
}
