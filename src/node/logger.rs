use chrono::prelude::Utc;
use serde::{self, Deserialize, Serialize};
use std::io::{self};
use std::sync::{Mutex, PoisonError};

#[derive(Default, Serialize, Deserialize)]
pub struct Logger {
    logs: Mutex<Vec<String>>,
    temp_logs: Mutex<Vec<String>>,
}

#[derive(Debug)]
pub enum LoggerError {
    PoisonError(String),
    FileWriteError(std::io::Error),
}
impl<T> From<PoisonError<T>> for LoggerError {
    fn from(value: PoisonError<T>) -> LoggerError {
        LoggerError::PoisonError(value.to_string())
    }
}
impl From<std::io::Error> for LoggerError {
    fn from(value: std::io::Error) -> Self {
        LoggerError::FileWriteError(value)
    }
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
            dt.format("%Y-%m-%d %H:%M:%S"),
            log
        ));
        self.temp_log(format!(
            "[ERROR] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S"),
            log
        ));
    }
    pub fn log(&self, log: String) {
        let dt = Utc::now();
        self.log_internal(format!(
            "[LOG] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S"),
            log
        ));
        self.temp_log(format!(
            "[LOG] {} | {}",
            dt.format("%Y-%m-%d %H:%M:%S"),
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

    // std::sync::PoisonError<MutexGuard<Vec<String>>>>
    pub fn write_logs_file(
        &self,
        path: &std::path::PathBuf,
    ) -> std::result::Result<(), LoggerError> {
        let v = match self.logs.lock() {
            Ok(v) => v,
            Err(e) => {
                println!("Poisoned mutex (this is not good)");
                return Err(LoggerError::from(e));
            }
        };
        let contents = v.join("\n");
        std::fs::write(path, contents)?;
        Ok(())
    }
    pub fn read_logs_file(path: &std::path::PathBuf) -> std::result::Result<Logger, LoggerError> {
        println!("config log path is {}", path.to_str().unwrap());
        let contents = std::fs::read_to_string(path)?;
        let logs = contents.split("\n").map(|str| str.to_string()).collect();

        Ok(Logger {
            logs: Mutex::new(logs),
            temp_logs: Mutex::new(Vec::new()),
        })
    }
}
// impl Serialize for Logger {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let v = match self.logs.lock() {
//             Ok(v) => v.join("\n"),
//             Err(e) => return Err(serde::ser::Error::custom(e.to_string())),
//         };
//         serializer.serialize_str(v.as_str())
//     }
// }
// struct LoggerVisitor;
// impl<T> serde::de::Visitor for LoggerError<T> {
//     fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Bool(v),
//             &self,
//         ))
//     }
//
//     fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_i64(v as i64)
//     }
//
//     fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_i64(v as i64)
//     }
//
//     fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_i64(v as i64)
//     }
//
//     fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Signed(v),
//             &self,
//         ))
//     }
//
//     fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         let mut buf = [0u8; 58];
//         let mut writer = format::Buf::new(&mut buf);
//         std::fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as i128", v)).unwrap();
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Other(writer.as_str()),
//             &self,
//         ))
//     }
//
//     fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_u64(v as u64)
//     }
//
//     fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_u64(v as u64)
//     }
//
//     fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_u64(v as u64)
//     }
//
//     fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Unsigned(v),
//             &self,
//         ))
//     }
//
//     fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         let mut buf = [0u8; 57];
//         let mut writer = format::Buf::new(&mut buf);
//         std::fmt::Write::write_fmt(&mut writer, format_args!("integer `{}` as u128", v)).unwrap();
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Other(writer.as_str()),
//             &self,
//         ))
//     }
//
//     fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_f64(v as f64)
//     }
//
//     fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Float(v),
//             &self,
//         ))
//     }
//
//     fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_str(v.encode_utf8(&mut [0u8; 4]))
//     }
//
//     fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Str(v),
//             &self,
//         ))
//     }
//
//     fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_str(v)
//     }
//
//     fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_str(&v)
//     }
//
//     fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Bytes(v),
//             &self,
//         ))
//     }
//
//     fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_bytes(v)
//     }
//
//     fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         self.visit_bytes(&v)
//     }
//
//     fn visit_none<E>(self) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Option,
//             &self,
//         ))
//     }
//
//     fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let _ = deserializer;
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Option,
//             &self,
//         ))
//     }
//
//     fn visit_unit<E>(self) -> Result<Self::Value, E>
//     where
//         E: serde::de::Error,
//     {
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Unit,
//             &self,
//         ))
//     }
//
//     fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let _ = deserializer;
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::NewtypeStruct,
//             &self,
//         ))
//     }
//
//     fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: serde::de::SeqAccess<'de>,
//     {
//         let _ = seq;
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Seq,
//             &self,
//         ))
//     }
//
//     fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
//     where
//         A: serde::de::MapAccess<'de>,
//     {
//         let _ = map;
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Map,
//             &self,
//         ))
//     }
//
//     fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
//     where
//         A: serde::de::EnumAccess<'de>,
//     {
//         let _ = data;
//         Err(serde::de::Error::invalid_type(
//             serde::de::Unexpected::Enum,
//             &self,
//         ))
//     }
// }
// impl<'de> Deserialize<'de> for Logger {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         deserializer.deserialize_str(LoggerVisitor)
//     }
// }
