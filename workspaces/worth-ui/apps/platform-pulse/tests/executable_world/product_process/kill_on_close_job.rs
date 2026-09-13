use std::os::windows::io::AsRawHandle;
use std::process::Child;

pub(super) struct KillOnCloseJob {
    job: win32job::Job,
}

impl KillOnCloseJob {
    pub(super) fn assign(child: &Child) -> Result<Self, String> {
        let job = win32job::Job::create().map_err(|error| error.to_string())?;
        let mut limits = job
            .query_extended_limit_info()
            .map_err(|error| error.to_string())?;
        limits.limit_kill_on_job_close();
        job.set_extended_limit_info(&limits)
            .map_err(|error| error.to_string())?;
        job.assign_process(child.as_raw_handle() as isize)
            .map_err(|error| error.to_string())?;
        Ok(Self { job })
    }

    pub(super) fn process_count(&self) -> Result<usize, String> {
        self.job
            .query_process_id_list()
            .map(|processes| processes.len())
            .map_err(|error| error.to_string())
    }
}

#[test]
fn kill_on_close_job_terminates_child_when_primary_termination_is_skipped() {
    let mut child = std::process::Command::new("cmd")
        .args(["/c", "ping -n 30 127.0.0.1 > nul"])
        .spawn()
        .expect("fixture child starts");
    let job = KillOnCloseJob::assign(&child).expect("fixture child enters kill-on-close job");
    assert_eq!(job.process_count().unwrap(), 1);
    drop(job);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "job close left the fixture child alive"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
