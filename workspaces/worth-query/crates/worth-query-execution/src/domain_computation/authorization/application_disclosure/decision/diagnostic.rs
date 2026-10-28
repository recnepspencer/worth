//! Fresh field and observable-influence admission precedes retained-body lookup.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionTarget;
use std::sync::Arc;
use worth_execution::{MapKernelContext, MapKernelFailure, MapKernelStop};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryDisclosureSelector, ApplicationQueryObservableInfluence,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledGraphReadContract,
};

/// A scoped loan of the actual session-bound current diagnostic governance.
/// Only this owner can construct it; it carries no copied permission or payload.
pub(in crate::domain_computation) struct DiagnosticComparisonAdmission<'a> {
    _governance: &'a WorthQueryApplicationQueryGovernance,
    graph: &'a WorthQueryInstalledGraphReadContract,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum DiagnosticComparisonDenial {
    NotDeclared,
    UnsupportedShape,
    ForeignBinding,
    FieldDenied,
    RelationDenied,
    InternalInfluenceDenied,
}

impl WorthQueryApplicationQueryGovernance {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn admit_diagnostic_comparison<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Scope,
    >(
        &'a self,
        context: &mut MapKernelContext<'_, '_>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        session: &WorthQueryManagedGraphWorkSession,
        target: &Arc<WorthQueryApplicationInspectionTarget>,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> Result<DiagnosticComparisonAdmission<'a>, MapKernelFailure<DiagnosticComparisonDenial>>
    {
        use DiagnosticComparisonDenial as Denial;
        if query.binding_observation() != Some(target.contract().binding_observation())
            || !self.inspection_matches(session, target, parameters, principal, scope)
            || !self.authorization_belongs_to_session(session.identity())
        {
            return Err(MapKernelFailure::Domain(Denial::ForeignBinding));
        }
        if !super::super::contract::diagnostic_shape_supported(query) {
            return Err(MapKernelFailure::Domain(Denial::UnsupportedShape));
        }
        let Self::Governed {
            computation,
            disclosure,
            ..
        } = self
        else {
            return Err(MapKernelFailure::Domain(Denial::NotDeclared));
        };
        let lookup_work = slot_lookup_work(query, context)?;
        for field in query.read_graph().projections() {
            checkpoint(context, lookup_work).map_err(MapKernelFailure::Stop)?;
            let Some(rule) = disclosure.disclosed.get(field.slot_key_identity().as_ref()) else {
                return Err(MapKernelFailure::Domain(Denial::FieldDenied));
            };
            if !diagnostic_rule(
                context,
                rule,
                (field.entity(), field.aspect(), field.field()),
            )? {
                return Err(MapKernelFailure::Domain(Denial::FieldDenied));
            }
        }
        for relation in query.read_graph().relations() {
            checkpoint(context, lookup_work).map_err(MapKernelFailure::Stop)?;
            if !disclosure
                .disclosed
                .get(relation.slot_key_identity().as_ref())
                .is_some_and(|rule| {
                    rule.influence()
                        .permits(ApplicationQueryObservableInfluence::Explanation)
                })
            {
                return Err(MapKernelFailure::Domain(Denial::RelationDenied));
            }
        }
        // The existing compiler verifies coverage of every predicate, guard and
        // ordering input. Their actual admitted rules must additionally allow
        // diagnostic explanation: output-field permission cannot cover them.
        for (field, rules) in &computation.internal_field_rules {
            for rule in rules {
                if !diagnostic_rule(context, rule, (&field.0, &field.1, &field.2))? {
                    return Err(MapKernelFailure::Domain(Denial::InternalInfluenceDenied));
                }
            }
        }
        Ok(DiagnosticComparisonAdmission {
            _governance: self,
            graph: query.read_graph(),
        })
    }
}

impl DiagnosticComparisonAdmission<'_> {
    pub(in crate::domain_computation) fn graph(&self) -> &WorthQueryInstalledGraphReadContract {
        self.graph
    }
}

fn diagnostic_rule(
    context: &mut MapKernelContext<'_, '_>,
    rule: &AdmittedDisclosureRule,
    expected: (&str, &str, &str),
) -> Result<bool, MapKernelFailure<DiagnosticComparisonDenial>> {
    let mut work = expected
        .0
        .len()
        .checked_add(expected.1.len())
        .and_then(|n| n.checked_add(expected.2.len()))
        .ok_or_else(overflow)?;
    if let Some(field) = rule.field() {
        work = work
            .checked_add(field.diagnostic_mask().owned_allocation_capacity_bytes())
            .ok_or_else(overflow)?;
    }
    checkpoint(context, u64::try_from(work).map_err(|_| overflow())?)
        .map_err(MapKernelFailure::Stop)?;
    Ok(rule
        .influence()
        .permits(ApplicationQueryObservableInfluence::Explanation)
        && rule.field().is_some_and(|field| {
            field.matches(expected)
                && field
                    .diagnostic_mask()
                    .paths()
                    .iter()
                    .any(|path| matches!(path.fields(), [key] if key.as_str() == expected.2))
        }))
}

fn slot_lookup_work<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<DiagnosticComparisonDenial>> {
    let mut width = std::mem::size_of::<ApplicationQueryResultSlotKey>();
    for rule in query.disclosure().rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let texts: [&str; 7] = match rule.selector() {
            ApplicationQueryDisclosureSelector::Field {
                query_type,
                slot_type,
                entity,
                aspect,
                field,
                output_name,
                value_type,
                ..
            } => [
                query_type.as_str(),
                slot_type.as_str(),
                entity,
                aspect,
                field,
                output_name,
                value_type.as_str(),
            ],
            ApplicationQueryDisclosureSelector::Relation {
                query_type,
                slot_type,
                relation,
                from,
                to,
                output_name,
                ..
            } => [
                query_type.as_str(),
                slot_type.as_str(),
                relation,
                from,
                to,
                output_name,
                "",
            ],
            ApplicationQueryDisclosureSelector::InternalField { .. } => continue,
            _ => {
                return Err(MapKernelFailure::Domain(
                    DiagnosticComparisonDenial::UnsupportedShape,
                ))
            }
        };
        let text_width = texts
            .into_iter()
            .try_fold(
                std::mem::size_of::<ApplicationQueryResultSlotKey>(),
                |bytes, text| bytes.checked_add(text.len()),
            )
            .ok_or_else(overflow)?;
        width = width.max(text_width);
    }
    let count = query
        .read_graph()
        .projections()
        .len()
        .checked_add(query.read_graph().relations().len())
        .ok_or_else(overflow)?;
    // A portable lookup bound visits no more than this actual map's entries
    // plus one, without depending on standard-library node width or height.
    u64::try_from(width)
        .ok()
        .and_then(|width| width.checked_mul(u64::try_from(count).ok()?.checked_add(1)?))
        .ok_or_else(overflow)
}

fn overflow() -> MapKernelFailure<DiagnosticComparisonDenial> {
    MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow)
}
