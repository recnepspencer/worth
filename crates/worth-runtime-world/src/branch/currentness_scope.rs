//! A short owner-issued read scope over one exact product-reference image.

use super::{ProductBranchObservation, ProductBranchReferenceCell};

/// Binds an issued observation to its live cell without acquiring new history
/// or component pins. Currentness is established only inside `while_current`.
#[must_use = "a currentness scope must enter its guarded section or be dropped"]
pub(crate) struct ProductBranchCurrentnessScope<'selected> {
    cell: ProductBranchReferenceCell,
    expected: &'selected ProductBranchObservation,
    _operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
    control: Option<
        std::sync::Arc<
            crate::lifecycle::operation_control_for_test::ProductCurrentnessLatchForTest,
        >,
    >,
}

/// Exact expected-head currentness, valid only during the owner's guarded
/// section. It grants no component mutation or product publication authority.
/// The witness cannot escape its guarded callback:
///
/// ```compile_fail
/// use worth_runtime_world::facade::{CurrentProductHead, ProductBranchObservation, RuntimeWorldObservationPort};
/// fn escape<'a>(port: &RuntimeWorldObservationPort, expected: &'a ProductBranchObservation) -> CurrentProductHead<'a> {
///     port.while_product_branch_current(expected, (), |(), head| head).unwrap()
/// }
/// ```
pub struct CurrentProductHead<'guard> {
    expected: &'guard ProductBranchObservation,
}

/// A displaced or retired expected head enters no guarded section and returns
/// the caller's prepared custody unchanged.
#[derive(Debug)]
pub enum ProductBranchCurrentnessFailure<Argument> {
    AdmissionDenied {
        denial:
            crate::lifecycle::RuntimeWorldServiceDenial<super::RuntimeWorldBranchAdmissionDenial>,
        argument: Argument,
    },
    ExpectedHeadUnavailable(Argument),
    /// The caller's admitted Work refused before a World registry read.
    PreparationDenied(Argument),
}

impl<'selected> ProductBranchCurrentnessScope<'selected> {
    pub(crate) fn owner_issued(
        cell: ProductBranchReferenceCell,
        expected: &'selected ProductBranchObservation,
        operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
        control: Option<
            std::sync::Arc<
                crate::lifecycle::operation_control_for_test::ProductCurrentnessLatchForTest,
            >,
        >,
    ) -> Self {
        Self {
            cell,
            expected,
            _operation: operation,
            control,
        }
    }

    /// Execute already-prepared derived-state installation while the exact
    /// selected head cannot move. Allocate and admit resources before calling.
    /// The callback must not access this product cell or call its World owner;
    /// return retired values so their cleanup occurs after the guard drops.
    pub(crate) fn while_current<Argument, Output>(
        self,
        argument: Argument,
        install: impl for<'guard> FnOnce(Argument, CurrentProductHead<'guard>) -> Output,
    ) -> Result<Output, ProductBranchCurrentnessFailure<Argument>> {
        let callback = |argument| {
            install(
                argument,
                CurrentProductHead {
                    expected: self.expected,
                },
            )
        };
        let result = match self.control {
            None => self.cell.while_current(self.expected, argument, callback),
            Some(control) => {
                let hold = control.hold_reference_for_test(self.cell.clone());
                let result = self.cell.while_current_controlled_for_test(
                    self.expected,
                    argument,
                    callback,
                    &control,
                );
                drop(hold);
                result
            }
        };
        result.map_err(|(_, argument)| {
            ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(argument)
        })
    }
}

impl CurrentProductHead<'_> {
    /// Six identity axes plus initialized branch-name comparison bytes.
    /// Admit this bound before entering the owner's guarded section.
    pub fn comparison_work_bound(expected: &ProductBranchObservation) -> usize {
        6 + expected.branch_identity().name().as_str().len()
    }

    pub fn observation(&self) -> &ProductBranchObservation {
        self.expected
    }
}
