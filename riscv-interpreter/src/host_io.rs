#[derive(Default)]
pub struct HostIO {
    pub input_stream: Vec<u8>,
    pub input_cursor: usize,
    capture_output: bool,
    stdout_stream: Vec<u8>,
    stderr_stream: Vec<u8>,
}

impl HostIO {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_input_stream(&mut self, input: Vec<u8>) {
        self.input_stream = input;
        self.input_cursor = 0;
    }

    pub fn set_capture_output(&mut self, capture_output: bool) {
        self.capture_output = capture_output;
        self.stdout_stream.clear();
        self.stderr_stream.clear();
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout_stream
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr_stream
    }

    pub fn write_stdout(&mut self, bytes: &[u8]) {
        if self.capture_output {
            self.stdout_stream.extend_from_slice(bytes);
        } else {
            print!("{}", String::from_utf8_lossy(bytes));
        }
    }

    pub fn write_stderr(&mut self, bytes: &[u8]) {
        if self.capture_output {
            self.stderr_stream.extend_from_slice(bytes);
        } else {
            eprintln!("{}", String::from_utf8_lossy(bytes));
        }
    }
}
