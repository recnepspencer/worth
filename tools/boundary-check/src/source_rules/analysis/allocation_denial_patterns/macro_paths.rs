//! A single marked-frame walk recognizes raw-kind reads in macro DSL tokens.
use proc_macro2::{Spacing, Span, TokenStream, TokenTree};

use super::raw_kind_types::RAW;

pub(super) enum Fragment<'a> {
    Mention(&'a syn::Path),
    Read(Span),
    Items(TokenStream),
}

struct Body {
    tokens: Vec<TokenTree>,
    index: usize,
    frames: Vec<Option<Span>>,
    path: syn::Path,
    path_mark: Option<Span>,
    within_angle: bool,
}

impl Body {
    fn new(tokens: Vec<TokenTree>, within_angle: bool) -> Self {
        Self {
            tokens,
            index: 0,
            frames: vec![None],
            path: syn::Path {
                leading_colon: None,
                segments: Default::default(),
            },
            path_mark: None,
            within_angle,
        }
    }

    fn clear_path(&mut self) {
        self.path.segments.clear();
        self.path_mark = None;
    }

    fn mark(&mut self, mark: Option<Span>) {
        let frame = self.frames.last_mut().unwrap();
        *frame = frame.or(mark);
    }
}

pub(super) fn scan(tokens: &[TokenTree], mut observe: impl FnMut(Fragment<'_>) -> bool) -> usize {
    let mut bodies = vec![Body::new(tokens.to_vec(), false)];
    let mut operations = 0;
    while let Some(body) = bodies.last_mut() {
        if body.index == body.tokens.len() {
            let body = bodies.pop().unwrap();
            operations += body.frames.len() - 1;
            let mark = body.frames.into_iter().flatten().next();
            if let Some(parent) = bodies.last_mut() {
                parent.mark(mark);
            }
            continue;
        }
        let index = body.index;
        body.index += 1;
        operations += 1;
        match body.tokens[index].clone() {
            TokenTree::Ident(name) => {
                // Build ident-only lexical prefixes, never argument syntax.
                // Alias resolution supplies meaning; the token walk only asks
                // whether this prefix mentions the raw kind.
                if index < 2 || !member(&body.tokens, index - 2) {
                    body.clear_path();
                }
                body.path.segments.push(syn::PathSegment {
                    ident: name.clone(),
                    arguments: syn::PathArguments::None,
                });
                operations += 1;
                if name == RAW || observe(Fragment::Mention(&body.path)) {
                    body.path_mark = body.path_mark.or(Some(name.span()));
                }
                body.mark(body.path_mark);
                if member(&body.tokens, index + 1) {
                    if let Some(mark) = body.path_mark {
                        observe(Fragment::Read(mark));
                    }
                }
            }
            TokenTree::Punct(p) if p.as_char() == '<' => {
                operations += 1;
                body.frames.push(body.path_mark);
                body.clear_path();
            }
            TokenTree::Punct(p) if p.as_char() == '>' && !arrow(&body.tokens, index) => {
                if body.frames.len() > 1 {
                    operations += 1;
                    let mark = body.frames.pop().unwrap();
                    body.mark(mark);
                    if member(&body.tokens, index + 1) {
                        if let Some(mark) = mark {
                            observe(Fragment::Read(mark));
                        }
                    }
                }
                body.clear_path();
            }
            TokenTree::Group(group) => {
                body.clear_path();
                let within_angle = body.within_angle || body.frames.len() > 1;
                // Complete Rust syntax keeps its existing scope-aware visitor.
                // A DSL group is entered here once, never sent back to scan.
                let tokens = group.stream();
                if within_angle || !observe(Fragment::Items(tokens.clone())) {
                    bodies.push(Body::new(tokens.into_iter().collect(), within_angle));
                }
            }
            TokenTree::Punct(p) if p.as_char() == ':' => {}
            _ => body.clear_path(),
        }
    }
    operations
}

fn member(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Punct(p)) if p.as_char() == ':')
        && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':')
}

fn arrow(tokens: &[TokenTree], index: usize) -> bool {
    index.checked_sub(1).is_some_and(|previous| {
        matches!(&tokens[previous], TokenTree::Punct(p)
            if p.as_char() == '-' && p.spacing() == Spacing::Joint)
    })
}

#[cfg(test)]
mod tests {
    use super::{scan, Fragment, RAW};

    #[test]
    fn doubling_a_macro_body_doubles_token_work() {
        for fragment in [
            "Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "a < b, Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "a < b < c, Family::<ExecutionAllocationDenialKind>::Cancelled: value;",
            "a < b, Family::<ExecutionAllocationDenialKind>::Cancelled > c => value;",
            "a < b, c > Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "Vec<u8> Family::<ExecutionAllocationDenialKind>::Cancelled => value;",
            "a < b, Family::<ExecutionAllocationDenialKind>::Cancelled,",
            "a < b, <Vec<ExecutionAllocationDenialKind> as IntoIterator>::Item,",
            "<fn() -> ExecutionAllocationDenialKind as Id>::Out,",
            "<Arr<ExecutionAllocationDenialKind, 3> as Id>::Out,",
            "wrap!({ nested!(Family::<ExecutionAllocationDenialKind>::Cancelled); }),",
        ] {
            let count = |n| {
                let tokens = fragment
                    .repeat(n)
                    .parse::<proc_macro2::TokenStream>()
                    .unwrap();
                let tokens: Vec<_> = tokens.into_iter().collect();
                let mut reads = 0;
                let work = scan(&tokens, |fragment| match fragment {
                    Fragment::Mention(path) => path.segments.last().is_some_and(|s| s.ident == RAW),
                    Fragment::Read(_) => {
                        reads += 1;
                        false
                    }
                    Fragment::Items(_) => false,
                });
                (work, reads)
            };
            let small = count(16);
            let large = count(32);
            assert_eq!(large.0, 2 * small.0, "{fragment}");
            assert_eq!((small.1, large.1), (16, 32), "{fragment}");
        }
    }
}
