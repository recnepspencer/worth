//! A distinct typed intent/result delegates to the common fresh source/query session.
use super::super::super::result::WorthQueryEvaluatedApplicationInspection;
use super::*;
pub struct WorthQueryEvaluatedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>(
    pub(super) WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>,
);
pub struct WorthQueryEvaluatedReferenceOnlyInspectionRequest<
    'a,
    'p,
    'r,
    'o,
    's,
    Schema,
    Binding,
    Disclosure,
>(WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>);
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryEvaluatedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
{
    pub fn reference_only(
        self,
    ) -> WorthQueryEvaluatedReferenceOnlyInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    > {
        WorthQueryEvaluatedReferenceOnlyInspectionRequest(self.0)
    }
}
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryEvaluatedReferenceOnlyInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    >
{
    pub fn with_allocation_policy<'funding>(
        self,
        policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryEvaluatedReferenceOnlyInspectionRequest<
        'funding,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    >
    where
        'a: 'funding,
    {
        WorthQueryEvaluatedReferenceOnlyInspectionRequest(
            WorthQueryConnectedReferenceOnlyInspectionRequest(self.0)
                .with_allocation_policy(policy)
                .0,
        )
    }
}
impl<'o, Schema, Binding, Disclosure>
    WorthQueryEvaluatedReferenceOnlyInspectionRequest<
        '_,
        '_,
        '_,
        'o,
        '_,
        Schema,
        Binding,
        Disclosure,
    >
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
    Disclosure: ApplicationSourceDisclosureBinding<Schema>,
{
    pub fn execute_with_diagnostics<QDC, QDO, QDI, SDC, SDO, SDI>(
        self,
        qd: Access<Schema, QDC, QDO, QDI>,
        sd: Access<Schema, SDC, SDO, SDI>,
    ) -> Result<WorthQueryEvaluatedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        QDI: ApplicationCapabilityRequest<Schema, QDC, Scope = QueryScope<Schema, Binding>>,
        SDI: ApplicationCapabilityRequest<Schema, SDC, Scope = SourceScope<Schema, Disclosure>>,
    {
        self.0
            .execute_with_diagnostics_core(qd, sd)
            .map(SourceInspectionResult::evaluation)
    }
    pub fn execute_governed<QMC, QMO, QMI, QDC, QDO, QDI, SMC, SMO, SMI, SDC, SDO, SDI>(
        self,
        qm: Access<Schema, QMC, QMO, QMI>,
        qd: Access<Schema, QDC, QDO, QDI>,
        sm: Access<Schema, SMC, SMO, SMI>,
        sd: Access<Schema, SDC, SDO, SDI>,
    ) -> Result<WorthQueryEvaluatedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        QMI: ApplicationCapabilityRequest<Schema, QMC, Scope = QueryScope<Schema, Binding>>,
        QDI: ApplicationCapabilityRequest<Schema, QDC, Scope = QueryScope<Schema, Binding>>,
        SMI: ApplicationCapabilityRequest<Schema, SMC, Scope = SourceScope<Schema, Disclosure>>,
        SDI: ApplicationCapabilityRequest<Schema, SDC, Scope = SourceScope<Schema, Disclosure>>,
    {
        self.0
            .execute_governed_core(qm, qd, sm, sd)
            .map(SourceInspectionResult::evaluation)
    }
}
