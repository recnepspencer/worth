use worth_signal::facade::ResourceCompletionBatchAdmissionReport;

fn duplicate_report(report: ResourceCompletionBatchAdmissionReport) {
    let _duplicate = report.clone();
}

fn main() {}
