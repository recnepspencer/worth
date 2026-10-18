//! The configuration facts the native topology fixture test build proves.
use syn::{punctuated::Punctuated, Attribute, Expr, Lit, Meta, Token};

pub(super) fn active(attributes: &[Attribute]) -> Result<bool, String> {
    let mut enabled = true;
    for attribute in attributes {
        if attribute.path().is_ident("cfg_attr") {
            return Err("cfg_attr is unsupported for compiled-example ownership".into());
        }
        if attribute.path().is_ident("cfg") {
            let predicate = attribute
                .parse_args::<Meta>()
                .map_err(|error| error.to_string())?;
            enabled &= evaluate(&predicate)?;
        }
    }
    Ok(enabled)
}

fn evaluate(predicate: &Meta) -> Result<bool, String> {
    match predicate {
        Meta::Path(path) if path.is_ident("test") => Ok(true),
        Meta::NameValue(value) if value.path.is_ident("target_family") => {
            if matches!(&value.value, Expr::Lit(value) if matches!(&value.lit, Lit::Str(value) if value.value() == "wasm"))
            {
                Ok(false)
            } else {
                Err("only the native target_family != wasm fact is supported".into())
            }
        }
        Meta::List(list) => {
            let predicates = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|error| error.to_string())?;
            // Resolve every operand: an unknown condition cannot become true under not().
            let values = predicates
                .iter()
                .map(evaluate)
                .collect::<Result<Vec<_>, _>>()?;
            if list.path.is_ident("all") {
                Ok(values.iter().all(|value| *value))
            } else if list.path.is_ident("any") {
                Ok(values.iter().any(|value| *value))
            } else if list.path.is_ident("not") && values.len() == 1 {
                Ok(!values[0])
            } else {
                Err("unsupported compiled-example cfg operator".into())
            }
        }
        _ => Err("unsupported compiled-example cfg predicate".into()),
    }
}
