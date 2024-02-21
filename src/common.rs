use async_compression::tokio::write::GzipEncoder;
use chrono::{DateTime, Datelike, Local, Timelike};
use env_logger::Env;
use log::info;
use regex::Regex;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use tokio::fs::{metadata, File};
use tokio::io::BufReader;
use tokio::process::Command;

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

// if file size < 10MB, don't compress
// don't do anything if an already compressed file is passed in
// Issue when used like file_backup_client.sh <filename> (w/o a path provided in filename, it fails)
/// compresses abs_file_or_dirname into system_tmp_dir/file_or_dirname.gz if a file or system_tmp_dir/file_or_dirname.tar.gz if a dir
pub async fn compress(
    abs_file_or_dirname: String,
    system_tmp_dir: String,
) -> Result<(String, String), std::io::Error> {
    let filedata = metadata(&abs_file_or_dirname).await?;
    let path = Path::new(&abs_file_or_dirname);
    // you can eliminate this sort of thing by using thiserror or anyhow crates
    let basefilename = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid filename"))?
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid filename"))?;

    if !abs_file_or_dirname.contains(".gz") {
        Ok((abs_file_or_dirname.clone(), basefilename.to_string()))
    } else if filedata.is_dir() {
        let archive_name = format!("{}.tar.gz", basefilename);
        let _abs_archive_path = format!("{}/{}", &system_tmp_dir, &archive_name);
        compress_dir_shell(&_abs_archive_path, &abs_file_or_dirname).await?;
        Ok((_abs_archive_path, archive_name))
    } else {
        let archive_name = format!("{}.gz", basefilename);
        let _abs_archive_path = format!("{}/{}", &system_tmp_dir, &archive_name);
        compress_file(&_abs_archive_path, &abs_file_or_dirname).await?;
        Ok((_abs_archive_path, archive_name))
    }
}

pub async fn compress_file(
    archive_name: &String,
    absfilename: &String,
) -> Result<(), std::io::Error> {
    let mut input_buf = BufReader::new(File::open(absfilename).await?);
    let archive_handle = File::create(archive_name).await?;

    let mut encoder = GzipEncoder::new(archive_handle);
    tokio::io::copy(&mut input_buf, &mut encoder).await?;
    Ok(())
}

pub async fn compress_dir_shell(
    archive_name: &String,
    absdirname: &String,
) -> Result<(), std::io::Error> {
    Command::new("tar")
        .arg("-zcf")
        .arg(archive_name)
        .arg(absdirname)
        .output()
        .await?;
    Ok(())
}
