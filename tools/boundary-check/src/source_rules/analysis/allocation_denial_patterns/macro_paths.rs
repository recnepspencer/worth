//! Recognize paths in macro DSL fragments, with disjoint token ownership.
use proc_macro2::{Spacing, TokenTree};

use super::raw_kind_types::RAW;

pub(super) enum Fragment {
    Path(syn::TypePath),
    Group(proc_macro2::TokenStream),
    RawMember(proc_macro2::Span),
}

pub(super) fn scan(tokens: &[TokenTree], mut path: impl FnMut(Fragment)) -> usize {
    let mut operations = 0;
    let mut candidate = Vec::new();
    let mut angles = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        operations += 1;
        if let TokenTree::Ident(name) = token {
            if name == RAW {
                operations += 2;
                if tokens.get(index + 1).is_some_and(colon)
                    && tokens.get(index + 2).is_some_and(colon)
                {
                    path(Fragment::RawMember(name.span()));
                }
            }
        }
        if let TokenTree::Group(group) = token {
            path(Fragment::Group(group.stream()));
        }
        let continues = match token {
            TokenTree::Group(_) if angles > 0 => true,
            TokenTree::Ident(_) => {
                if angles == 0 && matches!(candidate.last(), Some(TokenTree::Ident(_))) {
                    finish(&mut candidate, &mut path, &mut operations);
                }
                true
            }
            TokenTree::Punct(p) if p.as_char() == '<' => {
                angles += 1;
                true
            }
            TokenTree::Punct(p) if p.as_char() == '>' && angles > 0 => {
                angles -= 1;
                true
            }
            // Neither delimiter belongs to a type argument. In particular, an
            // unclosed comparison angle must not absorb the next DSL entry.
            TokenTree::Punct(p) if p.as_char() == ';' => false,
            TokenTree::Punct(p)
                if p.as_char() == '='
                    && tokens.get(index + 1).is_some_and(
                        |next| matches!(next, TokenTree::Punct(p) if p.as_char() == '>'),
                    ) =>
            {
                false
            }
            TokenTree::Punct(_) if angles > 0 => true,
            TokenTree::Punct(_) if colon(token) => path_colon(tokens, index),
            _ => false,
        };
        if continues {
            candidate.push(token.clone());
        } else {
            finish(&mut candidate, &mut path, &mut operations);
            angles = 0;
        }
    }
    finish(&mut candidate, &mut path, &mut operations);
    operations
}

fn finish(candidate: &mut Vec<TokenTree>, path: &mut impl FnMut(Fragment), operations: &mut usize) {
    // Each candidate is disjoint, so parsing cannot repeatedly reread prefixes.
    if candidate.iter().any(colon) {
        *operations += candidate.len();
        let tokens = std::mem::take(candidate).into_iter().collect();
        if let Ok(candidate) = syn::parse2::<syn::TypePath>(tokens) {
            path(Fragment::Path(candidate));
        }
    } else {
        candidate.clear();
    }
}

fn colon(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Punct(p) if p.as_char() == ':')
}

fn path_colon(tokens: &[TokenTree], index: usize) -> bool {
    matches!(&tokens[index], TokenTree::Punct(p) if p.spacing() == Spacing::Joint)
        && tokens.get(index + 1).is_some_and(colon)
        || index.checked_sub(1).is_some_and(|previous| {
            matches!(&tokens[previous], TokenTree::Punct(p)
                if p.as_char() == ':' && p.spacing() == Spacing::Joint)
        })
}

#[cfg(test)]
mod tests {
    use super::scan;

    #[test]
    fn doubling_a_macro_body_doubles_token_work() {
        let fragment = "Family::<ExecutionAllocationDenialKind>::Cancelled => value;";
        let count = |n| {
            let tokens = fragment
                .repeat(n)
                .parse::<proc_macro2::TokenStream>()
                .unwrap();
            let tokens: Vec<_> = tokens.into_iter().collect();
            let mut reads = 0;
            let work = scan(&tokens, |fragment| {
                if matches!(fragment, super::Fragment::Path(_)) {
                    reads += 1;
                }
            });
            (work, reads)
        };
        let small = count(16);
        let large = count(32);
        assert_eq!(large.0, 2 * small.0);
        assert_eq!((small.1, large.1), (16, 32));
    }
}
