//! String and Bytes builtins as resumable jobs: scalar counts, scalar
//! slices, containment, and affixes each charge one work unit per 8-byte
//! word they inspect, and a copied slice pays for its words before the copy.

use std::task::Poll;

use crate::expressions::denial::ExpressionResult;
use crate::expressions::operators::text::{Matcher, ScalarCount, ScalarRange, WORD};

use super::cost::words;
use super::jobs::{ready, Job};
use super::meter::EvaluationMeter;
use super::value::{ExpressionValue, Repr};

/// The bytes of a String or Bytes value.
fn bytes(value: &ExpressionValue) -> &[u8] {
    match &value.0 {
        Repr::String(text) => text.as_bytes(),
        Repr::Bytes(bytes) => bytes,
        _ => unreachable!("admission scans only String and Bytes"),
    }
}

#[derive(Debug)]
pub(super) enum TextJob {
    /// `length(string)`.
    Count {
        text: ExpressionValue,
        count: ScalarCount,
    },
    /// `slice(string, low, high)` while its byte range is found.
    Range {
        text: ExpressionValue,
        range: ScalarRange,
    },
    /// A found slice, paying for its words before it is copied.
    Copy {
        source: ExpressionValue,
        start: usize,
        end: usize,
        owed: u64,
    },
    Contains {
        haystack: ExpressionValue,
        needle: ExpressionValue,
        matcher: Matcher,
    },
    /// `starts_with` or `ends_with`, one word of the affix per step.
    Affix {
        text: ExpressionValue,
        affix: ExpressionValue,
        suffix: bool,
        offset: usize,
    },
}

impl TextJob {
    pub(super) fn count(text: ExpressionValue) -> Job {
        Job::Text(Box::new(Self::Count {
            text,
            count: ScalarCount::default(),
        }))
    }

    /// `None` when the bounds can never select a slice.
    pub(super) fn range(text: ExpressionValue, low: i128, high: i128) -> Option<Job> {
        let range = ScalarRange::new(low, high)?;
        Some(Job::Text(Box::new(Self::Range { text, range })))
    }

    /// Bytes `[start, end)` of `source`, which the caller has bounds-checked.
    pub(super) fn copy(source: ExpressionValue, start: usize, end: usize) -> Job {
        Job::Text(Box::new(Self::Copy {
            owed: words((end - start) as u64),
            source,
            start,
            end,
        }))
    }

    /// Allocates the needle's failure table before the scan starts.
    pub(super) fn contains(
        haystack: ExpressionValue,
        needle: ExpressionValue,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<Job> {
        meter.allocate(Matcher::table_bytes(bytes(&needle)))?;
        let matcher = Matcher::new(bytes(&needle));
        Ok(Job::Text(Box::new(Self::Contains {
            haystack,
            needle,
            matcher,
        })))
    }

    pub(super) fn affix(text: ExpressionValue, affix: ExpressionValue, suffix: bool) -> Job {
        Job::Text(Box::new(Self::Affix {
            text,
            affix,
            suffix,
            offset: 0,
        }))
    }

    pub(super) fn step(
        &mut self,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<Poll<ExpressionValue>> {
        loop {
            let found = match self {
                Self::Copy {
                    source,
                    start,
                    end,
                    owed,
                } => {
                    ready!(meter.prepay(owed));
                    let length = (*end - *start) as u64;
                    meter.allocate(length + 16)?;
                    meter.copied(length);
                    let slice = match &source.0 {
                        Repr::String(text) => ExpressionValue::string(&text[*start..*end]),
                        _ => ExpressionValue::bytes(&bytes(source)[*start..*end]),
                    };
                    meter.allocate(16)?;
                    return Ok(Poll::Ready(ExpressionValue::some(slice)));
                }
                _ if meter.work(1)?.is_pending() => return Ok(Poll::Pending),
                Self::Count { text, count } => match count.step(bytes(text)) {
                    Some(scalars) => return Ok(Poll::Ready(ExpressionValue::integer(scalars))),
                    None => continue,
                },
                Self::Range { text, range } => match range.step(bytes(text)) {
                    None => continue,
                    Some(None) => return Ok(Poll::Ready(ExpressionValue::none())),
                    Some(Some(found)) => (text.clone(), found),
                },
                Self::Contains {
                    haystack,
                    needle,
                    matcher,
                } => match matcher.step(bytes(haystack), bytes(needle)) {
                    Some(contained) => return Ok(Poll::Ready(ExpressionValue::bool(contained))),
                    None => continue,
                },
                Self::Affix {
                    text,
                    affix,
                    suffix,
                    offset,
                } => {
                    let (text, affix) = (bytes(text), bytes(affix));
                    if affix.len() > text.len() {
                        return Ok(Poll::Ready(ExpressionValue::bool(false)));
                    }
                    let base = if *suffix { text.len() - affix.len() } else { 0 };
                    let end = (*offset + WORD).min(affix.len());
                    if text[base + *offset..base + end] != affix[*offset..end] {
                        return Ok(Poll::Ready(ExpressionValue::bool(false)));
                    }
                    *offset = end;
                    if end == affix.len() {
                        return Ok(Poll::Ready(ExpressionValue::bool(true)));
                    }
                    continue;
                }
            };
            let (source, (start, end)) = found;
            *self = Self::Copy {
                owed: words((end - start) as u64),
                source,
                start,
                end,
            };
        }
    }
}
