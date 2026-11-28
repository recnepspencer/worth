use core::fmt::Write;

use super::{DenialClass, Disposition, OperatorReport, ReportDetail, RetryHint};

impl OperatorReport {
    /// Encodes the disclosed snapshot as compact UTF-8 JSON in fixed field order.
    /// The `worth.operator_report.v1` schema uses snake_case enum tags.
    pub fn to_canonical_json(&self) -> String {
        let mut json = String::from(r#"{"schema":"worth.operator_report.v1","denial_class":"#);
        push_string(&mut json, denial_tag(self.denial_class));
        json.push_str(r#","disposition":"#);
        push_disposition(&mut json, self.disposition);
        json.push_str(r#","report_code":"#);
        push_string(&mut json, self.report_code.as_str());
        json.push_str(r#","summary":"#);
        push_string(&mut json, self.summary);
        json.push_str(r#","detail":"#);
        push_detail(&mut json, &self.detail);
        json.push_str(r#","chain":["#);
        for (index, frame) in self.chain.iter().enumerate() {
            if index != 0 {
                json.push(',');
            }
            json.push_str(r#"{"detail":"#);
            push_detail(&mut json, &frame.detail);
            json.push('}');
        }
        json.push_str("]}");
        json
    }
}

fn denial_tag(class: DenialClass) -> &'static str {
    match class {
        DenialClass::InvalidInput => "invalid_input",
        DenialClass::NotFound => "not_found",
        DenialClass::Conflict => "conflict",
        DenialClass::Authorization => "authorization",
        DenialClass::Invariant => "invariant",
        DenialClass::RelationLaw => "relation_law",
        DenialClass::DeadlineElapsed => "deadline_elapsed",
        DenialClass::Cancelled => "cancelled",
        DenialClass::Exhausted => "exhausted",
        DenialClass::Unavailable => "unavailable",
        DenialClass::ExternalEffect => "external_effect",
        DenialClass::Indeterminate => "indeterminate",
        DenialClass::Defect => "defect",
    }
}

fn push_disposition(json: &mut String, disposition: Disposition) {
    let tag = match disposition {
        Disposition::Retry(hint) => {
            let hint = match hint {
                RetryHint::Immediate => "immediate",
                RetryHint::AfterBasisAdvance => "after_basis_advance",
                RetryHint::AfterBackoff => "after_backoff",
                RetryHint::WithFreshAuthority => "with_fresh_authority",
            };
            json.push_str(r#"{"kind":"retry","hint":"#);
            push_string(json, hint);
            json.push('}');
            return;
        }
        Disposition::CorrectInput => "correct_input",
        Disposition::Reauthorize => "reauthorize",
        Disposition::Escalate => "escalate",
        Disposition::Final => "final",
    };
    json.push_str(r#"{"kind":"#);
    push_string(json, tag);
    json.push('}');
}

fn push_detail(json: &mut String, detail: &ReportDetail) {
    match detail {
        ReportDetail::Withheld => json.push_str(r#"{"kind":"withheld"}"#),
        ReportDetail::Unavailable => json.push_str(r#"{"kind":"unavailable"}"#),
        ReportDetail::Disclosed(message) => {
            json.push_str(r#"{"kind":"disclosed","message":"#);
            push_string(json, message);
            json.push('}');
        }
    }
}

fn push_string(json: &mut String, value: &str) {
    json.push('"');
    for character in value.chars() {
        match character {
            '"' => json.push_str(r#"\""#),
            '\\' => json.push_str(r"\\"),
            '\u{08}' => json.push_str(r"\b"),
            '\u{0c}' => json.push_str(r"\f"),
            '\n' => json.push_str(r"\n"),
            '\r' => json.push_str(r"\r"),
            '\t' => json.push_str(r"\t"),
            '\u{00}'..='\u{1f}' => {
                write!(json, "\\u{:04x}", u32::from(character)).expect("writing to a String");
            }
            _ => json.push(character),
        }
    }
    json.push('"');
}
