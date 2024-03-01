use chrono::{DateTime, Datelike, Local, Timelike};
use env_logger::Env;
use log::info;
use regex::Regex;
use std::io::Write;
use std::io::{self};
use std::path::Path;

pub fn setup_logger() {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
    info!("=========================================================");
}

pub fn make_address_str(addr: &String, port: &i32) -> String {
    let a = String::clone(addr);
    a + ":" + &port.to_string()
}

/// used to format the filename that the server receives
pub fn format_filename(ip: &String, filename: &String) -> String {
    let now: DateTime<Local> = Local::now();
    let datetimestr = format!(
        "{:02}{:02}{:04}_{:02}{:02}{:02}",
        now.month(),
        now.day(),
        now.year(),
        now.hour(),
        now.minute(),
        now.second()
    );
    format!("{}__{}__{}", datetimestr, ip, filename)
}

/// Client needs to send filename in form: "filename:example_filename.zip:filename"
pub fn verify_filename(filename: String) -> Result<String, io::Error> {
    let re = std::cell::OnceCell::new();
    let re = re.get_or_init(|| Regex::new(r"filename:([a-zA-Z0-9\._-]+):filename").unwrap());

    if re.is_match(&filename) {
        let extracted_filename = filename.replace("filename:", "").replace(":filename", "");
        Ok(extracted_filename)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid message",
        ))
    }
}

pub fn get_fileinfo_to_send(filename: &str) -> Result<(String, String), io::Error> {
    let path = Path::new(filename);
    let archivename_to_tell_server = match path.is_dir() {
        true => path.file_name().unwrap().to_str().unwrap().to_string() + ".tar.gz",
        false => path.file_name().unwrap().to_str().unwrap().to_string() + ".gz",
    };
    let absolute_path_to_archive_and_send =
        std::fs::canonicalize(path)?.to_str().unwrap().to_string();

    Ok((
        absolute_path_to_archive_and_send,
        archivename_to_tell_server,
    ))
}