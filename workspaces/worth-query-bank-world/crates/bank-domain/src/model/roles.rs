#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum CustomerRole {
    PersonalOwner,
    BusinessOwner,
    Initiator,
    Approver,
    Viewer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmployeeRole {
    Teller,
    Auditor,
    BranchManager,
    EstateSpecialist,
    Compliance,
    Legal,
}
