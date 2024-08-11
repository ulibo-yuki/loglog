use chrono::{DateTime, Local};
use core::fmt;
use std::{
    env,
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    sync::OnceLock,
};

static CURRENT_DIR: OnceLock<PathBuf> = OnceLock::new();
static COLUMN: OnceLock<String> = OnceLock::new();

pub enum ErrorCode {
    FailedAddCsvColumn,
    FailedAddCsvData,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::FailedAddCsvColumn => write!(f, "failed add csv column."),
            ErrorCode::FailedAddCsvData => write!(f, "failed add csv data."),
        }
    }
}

pub struct LogStruct {
    date_time: DateTime<Local>,
    log_content: Vec<String>,
}

impl LogStruct {
    fn to_line(&self) -> String {
        format!(
            "\n{}{}\n",
            &self.date_time.format("%Y-%m-%d %H:%M:%S"),
            vec_to_string(&self.log_content),
        )
    }
}

fn vec_to_string(content: &Vec<String>) -> String {
    let mut content_csv = "".to_string();
    for i in content {
        content_csv.push(',');
        content_csv.push_str(i);
    }
    content_csv
}

fn add_csv_line(csv_path: &PathBuf, content: &String) -> Result<(), ErrorCode> {
    let file = match OpenOptions::new().append(true).open(csv_path) {
        Ok(file) => file,
        Err(_) => return Err(ErrorCode::FailedAddCsvData),
    };
    let mut bw = BufWriter::new(file);
    match bw.write_all(content.as_bytes()) {
        Ok(_) => match bw.flush() {
            Ok(_) => Ok(()),
            Err(_) => Err(ErrorCode::FailedAddCsvData),
        },
        Err(_) => Err(ErrorCode::FailedAddCsvData),
    }
}

fn add_csv_column(csv_path: PathBuf) -> Result<(), ErrorCode> {
    let file = match OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&csv_path)
    {
        Ok(file) => file,
        Err(_) => return Err(ErrorCode::FailedAddCsvColumn),
    };
    let column_str = format!("{}\n", COLUMN.get().expect("failed get path"));
    if let Some(line) = BufReader::new(file).lines().next() {
        match line {
            Ok(line) => {
                if line.trim() == column_str.trim() {
                    return Ok(());
                }
            }
            Err(_) => return Err(ErrorCode::FailedAddCsvColumn),
        }
    }
    match add_csv_line(&csv_path, &column_str) {
        Ok(_) => Ok(()),
        Err(error_code) => Err(error_code),
    }
}

fn to_csv_path() -> PathBuf {
    CURRENT_DIR
        .get()
        .expect("failed get current dir.")
        .join("log.csv")
}

pub fn loglog(log_content: Vec<String>) -> Result<(), ErrorCode> {
    let content = LogStruct {
        date_time: Local::now(),
        log_content,
    };
    match add_csv_column(to_csv_path()) {
        Ok(_) => match add_csv_line(&to_csv_path(), &content.to_line()) {
            Ok(_) => Ok(()),
            Err(error_code) => Err(error_code),
        },
        Err(error_code) => Err(error_code),
    }
}

pub fn set_column_exe_path(column: String) {
    let _ = COLUMN.set(
        format!(
            "{},{}",
            "date time",
            column,
        )
    );
    let _ = CURRENT_DIR.set(
        env::current_exe()
            .expect("failed get current_exe.")
            .parent()
            .expect("failed get parent.")
            .to_path_buf(),
    );
}

