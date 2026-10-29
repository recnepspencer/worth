pub(crate) use worth_query_worlds::facade::host::temporal_conditional_operation::advancement_custody::width_differential::*;

#[test]
fn wake_and_reconsideration_keep_bits_and_order_with_exact_width_work() {
    let host_request = worth_query_worlds::facade::host::test_request_policy::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    if let Ok(width) = std::env::var(WIDTH) {
        let placement = match width.as_str() {
            "0" => Placement::Serial,
            "1" => Placement::Leased(NonZeroUsize::MIN),
            "4" => Placement::Leased(NonZeroUsize::new(4).unwrap()),
            _ => panic!("the parent declares exactly three widths"),
        };
        capture(resource_request, placement);
        return;
    }
    let test = format!(
        "{}::wake_and_reconsideration_keep_bits_and_order_with_exact_width_work",
        module_path!().split_once("::").unwrap().1
    );
    let mut reference = None;
    let mut leased_reference = None;
    let mut serial_work = None;
    for width in ["0", "1", "4"] {
        let output = worth_query_worlds::facade::host::process_deadline::output(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &test, "--nocapture"])
                .env(WIDTH, width),
        )
        .expect("run the same real fixture with fresh live identities");
        assert!(
            output.status.success(),
            "width {width}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let actual = stdout
            .lines()
            .filter(|line| line.starts_with(DIGEST))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 2, "both actual semantic cases ran");
        println!("width {width}: {actual:?}");
        if let Some(reference) = &reference {
            assert_eq!(&actual, reference, "width {width}");
        } else {
            reference = Some(actual);
        }
        let actual_work = stdout
            .lines()
            .filter_map(|line| line.strip_prefix(LAST_WORK))
            .map(|line| {
                line.split(',')
                    .map(|units| units.parse::<u64>().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_work.len(), 2);
        if width == "0" {
            serial_work = Some(actual_work);
        } else {
            assert_eq!(
                &actual_work,
                serial_work.as_ref().unwrap(),
                "every request pays the same checked native work at width {width}"
            );
        }
        if width != "0" {
            let actual = stdout
                .lines()
                .filter(|line| line.starts_with(LEASED_DIGEST))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), 2, "both leased semantic cases ran");
            if let Some(reference) = &leased_reference {
                assert_eq!(&actual, reference, "leased width {width}");
            } else {
                leased_reference = Some(actual);
            }
        }
    }
}
