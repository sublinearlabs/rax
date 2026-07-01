use std::io::Read;

#[derive(Default)]
enum InputMode {
    #[default]
    Buffered,
    HostStdin,
}

#[derive(Default)]
pub struct HostIO {
    input_mode: InputMode,
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
        self.input_mode = InputMode::Buffered;
        self.input_stream = input;
        self.input_cursor = 0;
    }

    pub fn set_input_from_host(&mut self) {
        self.input_mode = InputMode::HostStdin;
        self.input_stream.clear();
        self.input_cursor = 0;
    }

    pub fn read_stdin(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        match self.input_mode {
            InputMode::Buffered => {
                let available_bytes = self.input_stream.len() - self.input_cursor;
                let bytes_to_read = std::cmp::min(len, available_bytes);
                let start = self.input_cursor;
                let end = start + bytes_to_read;
                self.input_cursor = end;

                Ok(self.input_stream[start..end].to_vec())
            }
            InputMode::HostStdin => {
                let mut input = vec![0; len];
                let bytes_read = std::io::stdin().read(&mut input)?;
                input.truncate(bytes_read);
                Ok(input)
            }
        }
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
