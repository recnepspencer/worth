//! Fixed semantic-owner order agrees with the decoded validation handoff.
use super::*;
pub(super) fn validation<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    rule: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
    touched: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
    projected: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    value.elements(request, |execution| {
        field(execution, "checks", request)?.elements(request, |check| {
            reported_rule(field(check, "rule", request)?, request, rule)?;
            let provenance = check.optional_field("custom_provenance", request)?;
            if !provenance.is_none() {
                touched(field(provenance, "touched", request)?)?;
            }
            let verdict = field(check, "verdict", request)?;
            if verdict.is_map() {
                verdict.fields(request, |variant, value| {
                    let violation = match variant {
                        "Violation" => value,
                        "Advisory" => field(value, "violation", request)?,
                        _ => return Err(malformed()),
                    };
                    violation_fields(
                        field(violation, "fields", request)?,
                        request,
                        rule,
                        projected,
                    )
                })?;
            }
            Ok(())
        })?;
        field(execution, "decisions", request)?.elements(request, |decision| {
            reported_rule(field(decision, "rule", request)?, request, rule)
        })?;
        Ok(())
    })?;
    Ok(())
}
fn reported_rule<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    rule: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    value.fields(request, |variant, value| {
        if variant == "Custom" {
            rule(field(value, "rule_id", request)?)?;
        }
        Ok(())
    })
}
fn violation_fields<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    rule: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
    projected: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    if !value.is_map() {
        return Ok(());
    }
    value.fields(request, |variant, value| {
        match variant {
            "CustomInvariantFailure" => {
                let identity = field(
                    field(value, "identity", request)?,
                    "semantic_identity",
                    request,
                )?;
                rule(field(identity, "rule_id", request)?)?;
            }
            "CustomInvariantViolation" => rule(field(
                field(value, "identity", request)?,
                "rule_id",
                request,
            )?)?,
            "DerivedRelationGraph" => {
                let violation = field(value, "violation", request)?;
                member(field(violation, "member", request)?, request, projected)?;
                let counterpart = violation.optional_field("counterpart", request)?;
                if !counterpart.is_none() {
                    member(counterpart, request, projected)?;
                }
            }
            _ => {}
        }
        Ok(())
    })
}
fn member<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    projected: &mut impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    value.fields(request, |variant, _| {
        if variant == "Projected" {
            projected(value)?;
        }
        Ok(())
    })
}
