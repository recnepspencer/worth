mod admission_order;
mod admit_one;
mod body_admission;
mod canonical_form;
mod diagnostics;
mod operand_resolution;
mod operand_types;
mod pending_expression;
mod use_sites;

use std::collections::{BTreeMap, BTreeSet};

use self::admission_order::ExpressionGraph;
use self::admit_one::admit_one;
use self::diagnostics::{declaration_diagnostic, malformed_declaration_diagnostic};
use self::operand_types::OperandScope;
pub(super) use self::pending_expression::PendingExpression;
use super::input_sealing::ExpressionInput;
use super::{
    WorthUiSealedExpression, WorthUiSemanticDeclaration, WorthUiSemanticPackageSealingState,
    WorthUiSemanticProvenanceRef,
};
use crate::semantic::WorthUiExpressionDeclaration;
use crate::source::{WorthUiDslCompileDiagnosticCode, WorthUiSourceModuleId};

impl WorthUiSemanticPackageSealingState {
    /// Holds a `condition` or `derived` node for package-level admission.
    pub(super) fn defer_expression(
        &mut self,
        input: ExpressionInput<'_>,
        provenance_ref: WorthUiSemanticProvenanceRef,
        module_id: &WorthUiSourceModuleId,
        insert_at: usize,
    ) {
        let ExpressionInput { block, introducer } = input;
        match WorthUiExpressionDeclaration::parse(block.name_text(), introducer, block.body_atoms())
        {
            Ok(declaration) => self.pending_expressions.push(PendingExpression {
                declaration,
                provenance: block.provenance().clone(),
                provenance_ref,
                module_id: module_id.clone(),
                insert_at,
            }),
            Err(error) => {
                self.refused_expressions
                    .insert(block.name_text().to_owned());
                self.diagnostics
                    .push(malformed_declaration_diagnostic(&error, block.provenance()));
            }
        }
    }

    /// Admits every deferred expression through the shared kernel and seals
    /// each into its module and the package expression table. Every failure
    /// is recorded; the caller fails the whole package on any diagnostic.
    pub(super) fn admit_expressions(&mut self) {
        let mut pending = std::mem::take(&mut self.pending_expressions);
        pending.sort_by(|left, right| {
            left.declaration
                .identity()
                .cmp(right.declaration.identity())
        });
        self.reject_duplicate_identities(&mut pending);
        let cycles = ExpressionGraph::new(&pending).cycles();
        for cycle in &cycles {
            let names: Vec<&str> = cycle
                .iter()
                .map(|position| pending[*position].declaration.identity())
                .collect();
            self.refused_expressions
                .extend(names.iter().map(|name| (*name).to_owned()));
            let entry = &pending[cycle[0]];
            self.diagnostics.push(declaration_diagnostic(
                WorthUiDslCompileDiagnosticCode::ExpressionCycle,
                format!(
                    "expression declarations form a cycle: {} -> {}",
                    names.join(" -> "),
                    names[0]
                ),
                &entry.provenance,
            ));
        }
        let scope = OperandScope {
            projections: self.projection_scope(),
            roles: pending
                .iter()
                .map(|expression| {
                    (
                        expression.declaration.identity(),
                        expression.declaration.role(),
                    )
                })
                .collect(),
        };
        let mut sealed = Vec::new();
        let mut diagnostics = Vec::new();
        let mut refused = Vec::new();
        for expression in &pending {
            match admit_one(expression, &scope) {
                Ok(found) => {
                    sealed.push((expression.module_id.clone(), expression.insert_at, found))
                }
                Err(mut found) => {
                    refused.push(expression.declaration.identity().to_owned());
                    diagnostics.append(&mut found);
                }
            }
        }
        self.diagnostics.append(&mut diagnostics);
        self.refused_expressions.extend(refused);
        self.insert_sealed_expressions(sealed);
    }

    fn reject_duplicate_identities(&mut self, pending: &mut Vec<PendingExpression>) {
        let mut seen = BTreeSet::new();
        let mut kept = Vec::with_capacity(pending.len());
        for expression in pending.drain(..) {
            let identity = expression.declaration.identity().to_owned();
            if !seen.insert(identity.clone()) {
                self.refused_expressions.insert(identity.clone());
                self.diagnostics.push(declaration_diagnostic(
                    WorthUiDslCompileDiagnosticCode::InvalidExpressionDeclaration,
                    format!("expression declaration `{identity}` appears more than once"),
                    &expression.provenance,
                ));
            } else {
                kept.push(expression);
            }
        }
        *pending = kept;
    }

    fn projection_scope(
        &self,
    ) -> BTreeMap<
        &str,
        (
            crate::source::WorthUiProjectionShape,
            crate::source::WorthUiProjectionNativeFamily,
        ),
    > {
        self.modules
            .values()
            .flat_map(|module| module.declarations.iter())
            .filter_map(|declaration| match declaration {
                WorthUiSemanticDeclaration::Projection(projection) => {
                    let requirement = projection.requirement();
                    Some((
                        requirement.declaration_identity(),
                        (requirement.shape(), requirement.native_family()),
                    ))
                }
                _ => None,
            })
            .collect()
    }

    /// Places each sealed expression among its module declarations.
    /// Expressions sort after every other declaration kind, so each module's
    /// expressions share one recorded position; the stable sort keeps them in
    /// identity order at that position, and a running count of earlier
    /// insertions offsets each one.
    fn insert_sealed_expressions(
        &mut self,
        mut sealed: Vec<(WorthUiSourceModuleId, usize, WorthUiSealedExpression)>,
    ) {
        sealed.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        let mut inserted: BTreeMap<WorthUiSourceModuleId, usize> = BTreeMap::new();
        for (module_id, insert_at, expression) in sealed {
            let earlier = inserted.entry(module_id.clone()).or_default();
            if let Some(module) = self.modules.get_mut(&module_id) {
                module.declarations.insert(
                    (insert_at + *earlier).min(module.declarations.len()),
                    WorthUiSemanticDeclaration::Expression(expression.clone()),
                );
            }
            *earlier += 1;
            self.expressions
                .insert(expression.identity().to_owned(), expression);
        }
    }
}
