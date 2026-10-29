pub(crate) use worth_query_bank_worlds::facade::server::parallel_history::transfer_history::*;

#[test]
fn seeded_transfers_match_at_host_policy_and_each_request_width() {
    if let Ok(width) = std::env::var("WORTH_BANK_HISTORY_WIDTH") {
        let width = width.parse::<usize>().unwrap();
        run(std::num::NonZeroUsize::new(width).map_or(Posture::Serial, Posture::Leased));
        return;
    }
    let mut baseline = None;
    let mut red = Vec::new();
    for width in [0, 1, 2, 5] {
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","process::server::parallel_history::transfer_history::seeded_transfers_match_at_host_policy_and_each_request_width","--nocapture"]).env("WORTH_BANK_HISTORY_WIDTH",width.to_string()).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "Bank width {width}: {stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let lines = stdout
            .lines()
            .filter(|line| line.starts_with("BANK_CELL:"))
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 5, "five observed transfer steps at each width");
        let records = stdout
            .lines()
            .filter(|s| s.starts_with("BANK_CELL:"))
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(expected) = &baseline {
            if expected != &records {
                red.push(format!("width {width}: {expected} versus {records}"));
            }
        } else {
            baseline = Some(records);
        }
    }
    for finding in &red {
        println!("BANK_NAMED_RED:{finding}");
    }
    assert!(
        red.is_empty(),
        "Bank results, charged work and commit sequence agree across request widths"
    );
}
