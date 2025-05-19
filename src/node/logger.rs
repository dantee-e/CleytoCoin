use chrono::prelude::Utc;
use std::io::{self};
use std::sync::Mutex;

pub struct Logger {
    logs: Mutex<Vec<String>>,
    temp_logs: Mutex<Vec<String>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
            temp_logs: Mutex::new(Vec::new()),
        }
    }

    fn temp_log(&self, log: String) {
        let mut temp_logs = self.temp_logs.lock().unwrap(); // Lock the Mutex to modify the temp_logs
        temp_logs.push(log);
        if temp_logs.len() > 50 {
            temp_logs.remove(0); // Remove the oldest log if there are more than 50
        }
    }

    fn log_internal(&self, log: String) {
        let mut logs = self.logs.lock().unwrap(); // Lock the Mutex to modify the logs
        logs.push(log);

        if let Err(e) = std::fs::write("/tmp/foo", logs.join("\n").as_bytes()) {
            eprintln!("Unable to write to log file: {e}");
        }
    }
    pub fn log_error(&self, log: String) {
        let dt = Utc::now();
        self.log_internal(format!(
            "[ERROR] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            log
        ));
        self.temp_log(format!(
            "[ERROR] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            log
        ));
    }
    pub fn log(&self, log: String) {
        let dt = Utc::now();
        self.log_internal(format!(
            "[LOG] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            log
        ));
        self.temp_log(format!(
            "[LOG] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            log
        ));
    }

    pub fn read_logs(&self) -> io::Result<Vec<String>> {
        let logs = self.logs.lock().unwrap();
        Ok(logs.clone())
    }
    pub fn read_temp_logs(&self) -> io::Result<Vec<String>> {
        let logs = self.temp_logs.lock().unwrap();
        Ok(logs.clone())
    }
}
