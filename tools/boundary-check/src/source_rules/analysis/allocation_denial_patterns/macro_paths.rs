//! Recognize paths in macro DSL fragments, with disjoint token ownership.
use proc_macro2::{Spacing, TokenTree};

use super::raw_kind_types::RAW;

pub(super) enum Fragment {
    Path(syn::TypePath),
    Group(proc_macro2::TokenStream),
    RawMember(proc_macro2::Span),
}

pub(super) fn scan(tokens: &[TokenTree], mut path: impl FnMut(Fragment)) -> usize {
    scan_fragments(tokens, &mut path, true)
}

fn scan_fragments(
    tokens: &[TokenTree],
    path: &mut impl FnMut(Fragment),
    first_pass: bool,
) -> usize {
    let mut operations = 0;
    let mut candidate = Vec::new();
    let mut angles = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        operations += 1;
        if first_pass {
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
        }
        let continues = match token {
            TokenTree::Group(_) if angles > 0 => true,
            TokenTree::Ident(_) => {
                if angles == 0 && matches!(candidate.last(), Some(TokenTree::Ident(_))) {
                    finish(&mut candidate, path, &mut operations, first_pass);
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
            finish(&mut candidate, path, &mut operations, first_pass);
            angles = 0;
        }
    }
    finish(&mut candidate, path, &mut operations, first_pass);
    operations
}

fn finish(
    candidate: &mut Vec<TokenTree>,
    path: &mut impl FnMut(Fragment),
    operations: &mut usize,
    first_pass: bool,
) {
    // Each candidate is disjoint, so parsing cannot repeatedly reread prefixes.
    if candidate.iter().any(colon) {
        *operations += candidate.len();
        let tokens = std::mem::take(candidate);
        match syn::parse2::<syn::TypePath>(tokens.iter().cloned().collect()) {
            Ok(candidate) => path(Fragment::Path(candidate)),
            Err(_) if first_pass => recover_failed_path(&tokens, path, operations),
            Err(_) => {}
        }
    } else {
        candidate.clear();
    }
}

// Failed paths may contain comparisons or adjacent generic types. Decide every
// angle boundary in one stack walk, then scan disjoint pieces once. A closing
// angle followed by :: stays with its path; other matched pairs split at both
// ends. Unmatched opens split too, so no comparison owns a following DSL entry.
fn recover_failed_path(
    tokens: &[TokenTree],
    path: &mut impl FnMut(Fragment),
    operations: &mut usize,
) {
    let mut unmatched = Vec::new();
    let mut boundaries = vec![false; tokens.len()];
    for (index, token) in tokens.iter().enumerate() {
        *operations += 1;
        match token {
            TokenTree::Punct(p) if p.as_char() == '<' => unmatched.push(index),
            TokenTree::Punct(p) if p.as_char() == '>' => {
                if let Some(open) = unmatched.pop() {
                    *operations += 2;
                    if !(tokens.get(index + 1).is_some_and(colon)
                        && tokens.get(index + 2).is_some_and(colon))
                    {
                        boundaries[open] = true;
                        boundaries[index] = true;
                    }
                }
            }
            _ => {}
        }
    }
    for open in unmatched {
        *operations += 1;
        boundaries[open] = true;
    }
    let mut start = 0;
    for (end, split) in boundaries.into_iter().enumerate() {
        *operations += 1;
        if split {
            // Reuse the callback type; groups and raw members belong to the
            // first pass. The recovered pieces never invoke recovery again.
            *operations += scan_fragments(&tokens[start..end], path, false);
            start = end + 1;
        }
    }
    *operations += scan_fragments(&tokens[start..], path, false);
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
        for fragment in [
            "Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "a < b, Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "a < b < c, Family::<ExecutionAllocationDenialKind>::Cancelled: value;",
            "a < b, Family::<ExecutionAllocationDenialKind>::Cancelled > c => value;",
            "a < b, c > Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "Vec<u8> Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            // No semicolon bounds these candidates: the recovery work itself
            // must scale linearly when the entire macro body doubles.
            "a < b, Family::<ExecutionAllocationDenialKind>::Cancelled,",
        ] {
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
}
