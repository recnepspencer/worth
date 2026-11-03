//! Exchange one genuine completion fact without changing its report or request.
use super::*;

impl WorthQueryInspectionReadCompletion {
    pub(in crate::domain_computation) fn exchange_report_fact(
        &mut self,
        other: &mut Self,
        fact: &str,
    ) {
        assert_ne!(self.managed_run, other.managed_run);
        assert_ne!(self.basis, other.basis);
        assert_eq!(
            self.release.resource_plan_identity(),
            other.release.resource_plan_identity(),
            "same provider label is intentionally insufficient"
        );
        assert_ne!(
            self.release.graph_reservation_identity(),
            other.release.graph_reservation_identity()
        );
        macro_rules! exchange {
            ($field:ident) => {
                std::mem::swap(&mut self.$field, &mut other.$field)
            };
        }
        match fact {
            "session" => exchange!(session),
            "managed_run" => exchange!(managed_run),
            "plan" => exchange!(plan),
            "target" => exchange!(target),
            "basis" => exchange!(basis),
            "basis_release" => exchange!(basis_release),
            "release" => exchange!(release),
            _ => panic!("unknown court fact"),
        }
    }
}
