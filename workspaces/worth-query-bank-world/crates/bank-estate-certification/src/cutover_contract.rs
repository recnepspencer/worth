//! Deleted Bank execution wrappers are absent even in otherwise-valid native-control contexts.
//!
//! ```compile_fail,E0432
//! use bank_server::BankAccountAccessExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankBusinessAccountCreationExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankMoneyMovementExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankMoneyMovementRetainedExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankPaymentDecisionExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankPaymentInitiationOutcome;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankPersonalAccountCreationExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! ```compile_fail,E0432
//! use bank_server::BankProgramMutationExecution;
//! use bank_domain::schema::BankSchema;
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! ```
//!
//! The current native audience is lawful; owning Bank construction uses its own result.
//! ```
//! use bank_server::BankOperationConstructionRefusal;
//! use bank_domain::schema::{BankSchema, SendMoney};
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationRequestExt};
//! fn native(request: &WorthQueryApplicationRequest<'_, '_, '_, BankSchema>) { let _ = request.advanced(); }
//! fn preserve(owner: BankOperationConstructionRefusal<SendMoney>) -> BankOperationConstructionRefusal<SendMoney> { owner }
//! ```
