//! Untrusted expression drafts from source text, the Rust builder, or the
//! draft codec. A draft has no execution entry; only admission reads it.

use super::admission::AdmissionContext;
use super::admitted::{admit, AdmittedExpression};
use super::compatibility::{decode_draft, encode_draft};
use super::denial::{ExpressionResource, ExpressionResult};
use super::functions::ExpressionFunctionCatalog;
use super::profile::{check_limit, AdmissionMeter, ExpressionProfile};
use super::syntax::ast::SyntaxTree;
use super::syntax::{parse_source, SyntaxLimits};
use super::types::{ExpressionSchema, ExpressionType};

/// Unchecked expression syntax. Reading it bounds structure; nothing is
/// typed, resolved, or trusted until admission.
///
/// A draft may be read under a wider profile than it is admitted under;
/// admission re-checks its input size, node count, and depth against the
/// admitting profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionDraft {
    tree: SyntaxTree,
    /// The metered input the draft was read from: source or encoded bytes.
    input: Option<(ExpressionResource, u64)>,
}

impl ExpressionDraft {
    /// Parses V1 source text under the profile's source, node, depth, and
    /// work ceilings.
    pub(crate) fn parse(source: &str, profile: &ExpressionProfile) -> ExpressionResult<Self> {
        let input = (ExpressionResource::SourceBytes, source.len() as u64);
        check_limit(profile, input.0, input.1)?;
        let limits = SyntaxLimits {
            max_nodes: saturating_u32(profile.limit(ExpressionResource::SyntaxNodes)),
            max_depth: u16::try_from(profile.limit(ExpressionResource::SyntaxDepth))
                .unwrap_or(u16::MAX),
        };
        let mut meter = AdmissionMeter::new(profile, ExpressionResource::AdmissionWork);
        let tree = parse_source(source, limits, &mut meter)?;
        Ok(Self {
            tree,
            input: Some(input),
        })
    }

    pub(crate) fn decode(bytes: &[u8], profile: &ExpressionProfile) -> ExpressionResult<Self> {
        let tree = decode_draft(bytes, profile)?;
        Ok(Self {
            tree,
            input: Some((ExpressionResource::DecodedBytes, bytes.len() as u64)),
        })
    }

    pub(crate) fn from_tree(tree: SyntaxTree) -> Self {
        Self { tree, input: None }
    }

    /// The versioned encoding of this draft, without source spans. Decoding
    /// it yields an equal draft with the same admitted identity.
    pub fn encode(&self) -> Vec<u8> {
        encode_draft(&self.tree)
    }

    /// Admits the draft against operand types, installed functions, and a
    /// resource profile.
    pub fn admit(
        &self,
        schema: &ExpressionSchema,
        catalog: &ExpressionFunctionCatalog,
        profile: ExpressionProfile,
    ) -> ExpressionResult<AdmittedExpression> {
        admit(
            self,
            AdmissionContext {
                schema,
                catalog,
                profile: &profile,
            },
            None,
        )
    }

    /// Admits the draft and requires its result to be `expected`.
    pub fn admit_as(
        &self,
        schema: &ExpressionSchema,
        catalog: &ExpressionFunctionCatalog,
        profile: ExpressionProfile,
        expected: &ExpressionType,
    ) -> ExpressionResult<AdmittedExpression> {
        admit(
            self,
            AdmissionContext {
                schema,
                catalog,
                profile: &profile,
            },
            Some(expected),
        )
    }

    /// Re-checks the structural ceilings the draft was read under against
    /// `profile`, which may be narrower.
    pub(crate) fn check_within(&self, profile: &ExpressionProfile) -> ExpressionResult<()> {
        if let Some((resource, used)) = self.input {
            check_limit(profile, resource, used)?;
        }
        check_limit(
            profile,
            ExpressionResource::SyntaxNodes,
            self.tree.len() as u64,
        )?;
        let depth = self.tree.root().map_or(0, |root| self.tree.depth(root));
        check_limit(profile, ExpressionResource::SyntaxDepth, u64::from(depth))
    }

    pub(crate) fn tree(&self) -> &SyntaxTree {
        &self.tree
    }
}

pub(crate) fn saturating_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
