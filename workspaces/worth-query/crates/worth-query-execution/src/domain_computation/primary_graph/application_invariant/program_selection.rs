//! Which rostered program speaks for one installed invariant on one candidate.
//!
//! A host may roster several programs over the same installed rule catalog. A
//! rule then governs only the candidates running under a program that declares
//! it; a candidate running under a peer program is not the rule's business and
//! is satisfied without evaluating it. Anything the adapter cannot attribute to
//! admitted meaning fails closed, because a rule that cannot tell whose
//! candidate it is has no standing to pass one.
//!
//! The activation read this module performs is the platform's, not the
//! author's. Relational charges it against the wrapped rule's own work meter,
//! so installation raises that meter by the platform reserve named in
//! `invariant_installation::program_activation_work_reserve` whenever a
//! descriptor carries program selection. The author therefore still
//! receives exactly the units they declared, and a budget exhausted on this
//! read is reported as the platform's own exhaustion rather than as an
//! unreadable record.

use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationInvariantDescriptor, WorthQueryProgramSupportEntry,
    WorthQueryProgramSupportRoster,
};
use worth_relational::facade::runtime::{
    CustomInvariantExecutionContext, CustomInvariantExecutionError, StructuralReadError,
};

use super::super::program_occurrence::{
    program_revision_rendering, WorthQueryProgramActivationCell,
};
use super::super::schema_layout::WorthQueryProgramActivationLayout;

/// Whether one installed rule speaks for the candidate in front of it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProgramRuleStanding {
    /// The active program declares this rule, so the rule decides the candidate.
    Governs,
    /// A peer rostered program is active, so this rule has nothing to say.
    Silent,
}

/// The rostered programs that declare one installed invariant, resolved against
/// the branch program activation record at evaluation time.
pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramRuleSelection {
    activation: WorthQueryProgramActivationCell,
    aspect: AspectKey,
    field: FieldKey,
    declaring: Box<[AspectValue]>,
    rostered: Box<[AspectValue]>,
}

impl WorthQueryProgramRuleSelection {
    pub(in crate::domain_computation::primary_graph) fn for_installed_rule<Schema>(
        descriptor: &WorthQueryInstalledApplicationInvariantDescriptor,
        roster: &WorthQueryProgramSupportRoster<Schema>,
        activation: WorthQueryProgramActivationCell,
        layout: &WorthQueryProgramActivationLayout,
    ) -> Result<Self, String> {
        let locator = &layout.program_revision_locator;
        let field = locator
            .field_path()
            .fields()
            .first()
            .cloned()
            .ok_or_else(|| "program activation revision locator names no field".to_owned())?;
        Ok(Self {
            activation,
            aspect: locator.aspect().aspect_key().clone(),
            field,
            declaring: roster
                .entries()
                .iter()
                .filter(|entry| declares(entry, descriptor))
                .map(|entry| program_revision_rendering(entry.revision()))
                .collect(),
            rostered: roster
                .entries()
                .iter()
                .map(|entry| program_revision_rendering(entry.revision()))
                .collect(),
        })
    }

    /// Reads the activation record out of the proposed state and decides
    /// whether the wrapped rule speaks for this candidate. The structural read
    /// costs one work unit, covered by the platform reserve installation added
    /// on top of the author's declared budget.
    pub(in crate::domain_computation::primary_graph) fn standing(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
    ) -> Result<WorthQueryProgramRuleStanding, CustomInvariantExecutionError> {
        verdict(
            &self.declaring,
            &self.rostered,
            self.active_rendering(context)?,
        )
    }

    fn active_rendering<'state>(
        &self,
        context: &CustomInvariantExecutionContext<'state>,
    ) -> Result<&'state AspectValue, CustomInvariantExecutionError> {
        let identity = self
            .activation
            .published()
            .ok_or_else(|| unattributable("branch program activation was never published"))?;
        let state = context
            .aspect_states()
            .entity_aspect_state(identity)
            .map_err(activation_read_refusal)?;
        let value = state
            .get(&self.aspect)
            .ok_or_else(|| unattributable("branch program activation carries no aspect"))?;
        let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
            return Err(unattributable(
                "branch program activation carries a foreign aspect shape",
            ));
        };
        fields
            .get(&self.field)
            .ok_or_else(|| unattributable("branch program activation carries no revision"))
    }
}

/// Names why the platform's own activation read produced no rendering.
///
/// Budget exhaustion is reported apart from an unreadable record: the read is
/// the platform's, paid for by the installation reserve, so exhausting the
/// meter here accuses the platform's own cost accounting rather than telling
/// the author their record could not be read.
fn activation_read_refusal(error: StructuralReadError) -> CustomInvariantExecutionError {
    match error {
        StructuralReadError::WorkBudgetExceeded => {
            unattributable("platform branch program activation read exhausted the rule work budget")
        }
        unreadable => unattributable(format!(
            "branch program activation is unreadable: {unreadable:?}"
        )),
    }
}

/// Decides standing from the activation rendering alone.
///
/// A rendering no rostered program answers to is refused rather than passed:
/// a rule that cannot tell whose candidate it is has no standing to satisfy
/// one.
fn verdict(
    declaring: &[AspectValue],
    rostered: &[AspectValue],
    rendering: &AspectValue,
) -> Result<WorthQueryProgramRuleStanding, CustomInvariantExecutionError> {
    if declaring.iter().any(|declared| declared == rendering) {
        return Ok(WorthQueryProgramRuleStanding::Governs);
    }
    if rostered.iter().any(|admitted| admitted == rendering) {
        return Ok(WorthQueryProgramRuleStanding::Silent);
    }
    Err(unattributable(
        "branch program activation names no rostered program",
    ))
}

/// Whether one rostered program declares the exact installed rule contract:
/// same stable identity, same declared version, same execution point.
fn declares(
    entry: &WorthQueryProgramSupportEntry,
    descriptor: &WorthQueryInstalledApplicationInvariantDescriptor,
) -> bool {
    entry.rules().iter().any(|rule| {
        rule.identity() == descriptor.identifier()
            && rule.major() == descriptor.major()
            && rule.minor() == descriptor.minor()
            && rule.execution_point() == descriptor.execution_point()
    })
}

fn unattributable(detail: impl Into<String>) -> CustomInvariantExecutionError {
    CustomInvariantExecutionError::new(detail.into())
}

#[cfg(test)]
mod tests {
    use super::{
        activation_read_refusal, verdict, StructuralReadError, WorthQueryProgramRuleStanding,
    };
    use worth_foundational::facade::{AspectValue, InternedString};

    fn rendering(text: &str) -> AspectValue {
        AspectValue::String(InternedString::from(text))
    }

    #[test]
    fn the_active_program_that_declares_the_rule_is_governed_by_it() {
        let declaring = [rendering("declaring-revision")];
        let rostered = [rendering("declaring-revision"), rendering("peer-revision")];
        assert_eq!(
            verdict(&declaring, &rostered, &rendering("declaring-revision"))
                .expect("a declaring program leaves the rule its standing"),
            WorthQueryProgramRuleStanding::Governs
        );
    }

    #[test]
    fn a_peer_rostered_program_leaves_the_rule_silent_without_evaluating_it() {
        let declaring = [rendering("declaring-revision")];
        let rostered = [rendering("declaring-revision"), rendering("peer-revision")];
        assert_eq!(
            verdict(&declaring, &rostered, &rendering("peer-revision"))
                .expect("a rostered peer is admitted meaning"),
            WorthQueryProgramRuleStanding::Silent
        );
    }

    #[test]
    fn an_unrostered_activation_fails_closed_rather_than_satisfying_the_rule() {
        let declaring = [rendering("declaring-revision")];
        let rostered = [rendering("declaring-revision"), rendering("peer-revision")];
        let refusal = verdict(&declaring, &rostered, &rendering("stranger-revision"))
            .expect_err("an unattributable activation must not decide a candidate");
        assert!(format!("{refusal:?}").contains("names no rostered program"));
    }

    #[test]
    fn an_empty_roster_can_attribute_nothing() {
        let refusal = verdict(&[], &[], &rendering("declaring-revision"))
            .expect_err("no roster can attribute no candidate");
        assert!(format!("{refusal:?}").contains("names no rostered program"));
    }

    #[test]
    fn budget_exhaustion_on_the_platform_read_accuses_the_platform() {
        let refusal = activation_read_refusal(StructuralReadError::WorkBudgetExceeded);
        assert!(format!("{refusal:?}")
            .contains("platform branch program activation read exhausted the rule work budget"));
    }

    #[test]
    fn a_record_the_platform_cannot_reach_is_still_reported_as_unreadable() {
        let refusal = activation_read_refusal(StructuralReadError::RecordUnavailable);
        let reported = format!("{refusal:?}");
        assert!(reported.contains("branch program activation is unreadable"));
        assert!(!reported.contains("exhausted"));
    }
}
