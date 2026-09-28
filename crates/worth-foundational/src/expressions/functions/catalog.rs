//! The immutable installed-function catalog.
//!
//! Function bodies are checked expression programs over typed parameters and
//! previously installed functions, so the call graph is acyclic by
//! construction. Each function carries its expanded instruction count and call
//! depth, so callers check expansion before anything is flattened.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::expressions::admission::{check_tree, AdmissionContext, Scope};
use crate::expressions::canonical::{derive_identity, ExpressionProgramIdentity, IdentityInputs};
use crate::expressions::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionResource, ExpressionResult, SyntaxDenial,
};
use crate::expressions::draft::ExpressionDraft;
use crate::expressions::profile::{check_limit, ExpressionProfile};
use crate::expressions::program::ExpressionProgram;
use crate::expressions::syntax::{ExpressionSourceMap, GENERIC_INTRINSICS};
use crate::expressions::types::{ExpressionSchema, ExpressionType, ExpressionTypeName};

/// One installed function: exact signature, checked body, and identity.
#[derive(Debug)]
pub struct InstalledExpressionFunction {
    name: ExpressionTypeName,
    parameters: Box<[(Box<str>, ExpressionType)]>,
    result: ExpressionType,
    program: ExpressionProgram,
    closure: Box<[Arc<InstalledExpressionFunction>]>,
    identity: Arc<ExpressionProgramIdentity>,
    expanded_instructions: u64,
    call_depth: u32,
    /// The widest bus in the body or any callee.
    bit_width: u32,
}

impl InstalledExpressionFunction {
    /// The qualified function name.
    pub fn name(&self) -> &str {
        self.name.name()
    }

    /// The semantic version; a meaning change is a new version.
    pub fn version(&self) -> u32 {
        self.name.version()
    }

    /// Parameter names and types in declared order.
    pub fn parameters(&self) -> &[(Box<str>, ExpressionType)] {
        &self.parameters
    }

    pub fn result(&self) -> &ExpressionType {
        &self.result
    }

    /// Canonical meaning of the body, including its function closure.
    pub fn identity(&self) -> &Arc<ExpressionProgramIdentity> {
        &self.identity
    }

    /// The body's origins in its own authored source.
    pub fn source_map(&self) -> ExpressionSourceMap {
        ExpressionSourceMap::new(
            self.program
                .nodes()
                .iter()
                .map(|node| node.origin)
                .collect(),
        )
    }

    /// Direct installed callees in canonical closure order.
    pub fn functions(&self) -> impl ExactSizeIterator<Item = &Arc<InstalledExpressionFunction>> {
        self.closure.iter()
    }

    pub(crate) fn expanded_instructions(&self) -> u64 {
        self.expanded_instructions
    }

    pub(crate) fn call_depth(&self) -> u32 {
        self.call_depth
    }

    pub(crate) fn bit_width(&self) -> u32 {
        self.bit_width
    }

    /// Canonical closure order: name, then version, then body digest.
    pub(crate) fn closure_key(&self) -> (&str, u32, &[u8; 32]) {
        (self.name(), self.version(), self.identity.digest_bytes())
    }
}

/// An immutable catalog of installed expression functions over one set of
/// type declarations. Expression source can select only what it contains.
#[derive(Debug, Clone)]
pub struct ExpressionFunctionCatalog {
    schema: ExpressionSchema,
    functions: Vec<Arc<InstalledExpressionFunction>>,
    by_name: BTreeMap<Box<str>, Vec<usize>>,
}

impl ExpressionFunctionCatalog {
    /// Starts a catalog over `schema`'s type declarations; its operands are
    /// not visible to function bodies.
    pub fn builder(
        schema: &ExpressionSchema,
        profile: ExpressionProfile,
    ) -> ExpressionFunctionCatalogBuilder {
        ExpressionFunctionCatalogBuilder {
            catalog: Self {
                schema: schema.declarations_only(),
                functions: Vec::new(),
                by_name: BTreeMap::new(),
            },
            profile,
        }
    }

    /// Installed functions in installation order.
    pub fn functions(&self) -> impl ExactSizeIterator<Item = &Arc<InstalledExpressionFunction>> {
        self.functions.iter()
    }

    pub(crate) fn schema(&self) -> &ExpressionSchema {
        &self.schema
    }

    pub(crate) fn overloads(&self, name: &str) -> impl Iterator<Item = usize> + '_ {
        self.by_name.get(name).into_iter().flatten().copied()
    }

    pub(crate) fn function(&self, index: usize) -> &Arc<InstalledExpressionFunction> {
        &self.functions[index]
    }
}

/// Installs functions one at a time, each admitted against those before it.
#[derive(Debug, Clone)]
pub struct ExpressionFunctionCatalogBuilder {
    catalog: ExpressionFunctionCatalog,
    profile: ExpressionProfile,
}

/// A function to install: qualified versioned name, exact signature, body.
#[derive(Debug, Clone)]
pub struct ExpressionFunctionDeclaration {
    pub name: ExpressionTypeName,
    pub parameters: Vec<(Box<str>, ExpressionType)>,
    pub result: ExpressionType,
    pub body: ExpressionDraft,
}

impl ExpressionFunctionCatalogBuilder {
    /// Admits and installs one function. A denial leaves the catalog as it was.
    pub fn install(&mut self, declaration: ExpressionFunctionDeclaration) -> ExpressionResult<()> {
        let ExpressionFunctionDeclaration {
            name,
            parameters,
            result,
            body,
        } = declaration;
        let catalog = &self.catalog;
        let profile = &self.profile;
        check_limit(
            profile,
            ExpressionResource::CatalogEntries,
            catalog.functions.len() as u64 + 1,
        )?;
        check_signature(catalog, &name, &parameters, &result)?;
        let context = AdmissionContext {
            schema: &catalog.schema,
            catalog,
            profile,
        };
        body.check_within(profile)?;
        let checked = check_tree(
            body.tree(),
            &context,
            Scope::Parameters(&parameters),
            Some(&result),
        )?;
        let closure: Box<[_]> = checked
            .functions
            .iter()
            .map(|index| catalog.functions[*index].clone())
            .collect();
        let identity = derive_identity(IdentityInputs {
            program: &checked.program,
            slots: parameters.iter().map(|(_, ty)| (None, ty)).collect(),
            closure: &closure,
            schema: &catalog.schema,
            profile,
        })?;
        let function = InstalledExpressionFunction {
            name,
            parameters: parameters.into_boxed_slice(),
            result,
            program: checked.program,
            closure,
            identity: Arc::new(identity),
            expanded_instructions: checked.expanded_instructions,
            call_depth: checked.call_depth,
            bit_width: checked.bit_width,
        };
        let index = self.catalog.functions.len();
        self.catalog
            .by_name
            .entry(function.name().into())
            .or_default()
            .push(index);
        self.catalog.functions.push(Arc::new(function));
        Ok(())
    }

    pub fn build(self) -> ExpressionFunctionCatalog {
        self.catalog
    }
}

/// Names are qualified; parameters are unique identifiers over declared
/// types; overloads differ in parameter types, never in version alone.
fn check_signature(
    catalog: &ExpressionFunctionCatalog,
    name: &ExpressionTypeName,
    parameters: &[(Box<str>, ExpressionType)],
    result: &ExpressionType,
) -> ExpressionResult<()> {
    if !name.name().contains("::") {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::UnsupportedFeature(
                "installed function names are qualified, such as `domain::function`",
            ),
        ));
    }
    let mut seen = BTreeSet::new();
    for (parameter, ty) in parameters {
        if !crate::expressions::types::is_identifier(parameter) {
            return Err(ExpressionDenial::new(ExpressionDenialDetail::Syntax(
                SyntaxDenial::InvalidIdentifier,
            )));
        }
        // Like schema operands, parameters cannot take generic intrinsic
        // names, which would read ambiguously as `name<...>(...)` calls.
        if !seen.insert(&**parameter) || GENERIC_INTRINSICS.contains(&&**parameter) {
            return Err(ExpressionDenial::new(
                ExpressionDenialDetail::AmbiguousBinding(parameter.to_string()),
            ));
        }
        catalog.schema.check_type(ty)?;
    }
    catalog.schema.check_type(result)?;
    let conflicting = catalog.overloads(name.name()).any(|index| {
        let existing = catalog.function(index).parameters();
        existing.len() == parameters.len()
            && existing
                .iter()
                .zip(parameters)
                .all(|((_, a), (_, b))| a == b)
    });
    if conflicting {
        return Err(ExpressionDenial::new(
            ExpressionDenialDetail::AmbiguousBinding(name.name().to_string()),
        ));
    }
    Ok(())
}
