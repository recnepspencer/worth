//! Passive NDJSON trace recorder for `tools/tla/run_tlc.py trace`.

use worth_query_host::facade::shared_access::ScopeAccess;

use super::dst_oracle::{DstOp, PassivePostState, PRINCIPALS, SCOPES};

#[derive(Default)]
pub struct TraceRecorder {
    lines: Vec<String>,
}

impl TraceRecorder {
    pub fn record(&mut self, op: DstOp, outcome: &str, value: u64, post: &PassivePostState) {
        let action_fields = format_action_fields(op);
        let w = |open: bool| if open { "Open" } else { "Closed" };
        let line = format!(
            "{{{action_fields},\"outcome\":\"{outcome}\",\"value\":{value},\
\"postHomeOpen\":{},\"postIncarnation\":{},\
\"postProject1Value\":{},\"postProject2Value\":{},\
\"postGrantAliceProject1\":\"{}\",\"postGrantAliceProject2\":\"{}\",\
\"postGrantBobProject1\":\"{}\",\"postGrantBobProject2\":\"{}\",\
\"postWatchAliceProject1\":\"{}\",\"postWatchAliceProject2\":\"{}\",\
\"postWatchBobProject1\":\"{}\",\"postWatchBobProject2\":\"{}\"}}",
            post.home_open,
            post.incarnation,
            post.project_values[0],
            post.project_values[1],
            post.grants[0][0].as_tla(),
            post.grants[0][1].as_tla(),
            post.grants[1][0].as_tla(),
            post.grants[1][1].as_tla(),
            w(post.watches_open[0][0]),
            w(post.watches_open[0][1]),
            w(post.watches_open[1][0]),
            w(post.watches_open[1][1]),
        );
        self.lines.push(line);
    }

    pub fn to_ndjson(&self) -> String {
        self.lines.join("\n") + "\n"
    }
}

fn format_action_fields(op: DstOp) -> String {
    match op {
        DstOp::OwnerWrite { scope } => {
            format!("\"action\":\"OwnerWrite\",\"scope\":\"{}\"", SCOPES[scope])
        }
        DstOp::GrantScopeAccess {
            principal,
            scope,
            access,
        } => {
            let acc = match access {
                ScopeAccess::ReadOnly => "ReadOnly",
                ScopeAccess::ReadAndWatch => "ReadAndWatch",
            };
            format!(
                "\"action\":\"GrantScopeAccess\",\"principal\":\"{}\",\"scope\":\"{}\",\"access\":\"{acc}\"",
                PRINCIPALS[principal], SCOPES[scope]
            )
        }
        DstOp::RevokeScopeAccess { principal, scope } => format!(
            "\"action\":\"RevokeScopeAccess\",\"principal\":\"{}\",\"scope\":\"{}\"",
            PRINCIPALS[principal], SCOPES[scope]
        ),
        DstOp::RemoteRead { principal, scope } => format!(
            "\"action\":\"RemoteRead\",\"principal\":\"{}\",\"scope\":\"{}\"",
            PRINCIPALS[principal], SCOPES[scope]
        ),
        DstOp::RemoteWatchOpen { principal, scope } => format!(
            "\"action\":\"RemoteWatchOpen\",\"principal\":\"{}\",\"scope\":\"{}\"",
            PRINCIPALS[principal], SCOPES[scope]
        ),
        DstOp::RemoteWatchPoll { principal, scope } => format!(
            "\"action\":\"RemoteWatchPoll\",\"principal\":\"{}\",\"scope\":\"{}\"",
            PRINCIPALS[principal], SCOPES[scope]
        ),
        DstOp::CloseHome => "\"action\":\"CloseHome\"".to_owned(),
        DstOp::ReopenHome => "\"action\":\"ReopenHome\"".to_owned(),
        DstOp::TamperAuthToken { principal } => format!(
            "\"action\":\"TamperAuthToken\",\"principal\":\"{}\"",
            PRINCIPALS[principal]
        ),
        DstOp::RestoreAuthToken { principal } => format!(
            "\"action\":\"RestoreAuthToken\",\"principal\":\"{}\"",
            PRINCIPALS[principal]
        ),
    }
}
