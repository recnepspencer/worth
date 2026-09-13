use std::io::{self, Read};
use std::thread::{self, JoinHandle};

pub(crate) struct NativeProcessOutputCapture {
    reader: Option<JoinHandle<io::Result<String>>>,
}

impl NativeProcessOutputCapture {
    pub(crate) fn start(mut stdout: std::process::ChildStdout) -> Self {
        let reader = thread::spawn(move || {
            let mut output = String::new();
            stdout.read_to_string(&mut output)?;
            Ok(output)
        });
        Self {
            reader: Some(reader),
        }
    }

    pub(crate) fn read_to_string(&mut self, output: &mut String) -> io::Result<usize> {
        let reader = self.reader.take().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "native output already consumed",
            )
        })?;
        let captured = reader
            .join()
            .map_err(|_| io::Error::other("native output reader panicked"))??;
        let length = captured.len();
        output.push_str(&captured);
        Ok(length)
    }
}
