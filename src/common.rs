use chrono::{DateTime, Datelike, Local, Timelike};
use env_logger::Env;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::debug;
use regex::Regex;
use std::fs::{metadata, File};
use std::io::Write;
use std::io::{self, BufReader};
use std::path::Path;
use std::process::Command;
use tar;

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
    let re = Regex::new(r"filename:([a-zA-Z0-9\._-]+):filename").unwrap();

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

/// compresses abs_file_or_dirname into system_tmp_dir/file_or_dirname.gz if a file or system_tmp_dir/file_or_dirname.tar.gz if a dir
pub fn compress(
    abs_file_or_dirname: String,
    system_tmp_dir: String,
) -> Result<(String, String), std::io::Error> {
    let filedata = metadata(&abs_file_or_dirname)?;
    let path = Path::new(&abs_file_or_dirname);
    let basefilename = path.file_name().unwrap().to_str().unwrap();

    if filedata.is_dir() {
        let archive_name = format!("{}.tar.gz", basefilename);
        let _abs_archive_path = format!("{}/{}", &system_tmp_dir, &archive_name);
        compress_dir_shell(&_abs_archive_path, &abs_file_or_dirname)?;
        Ok((_abs_archive_path, archive_name))
    } else {
        let archive_name = format!("{}.gz", basefilename);
        let _abs_archive_path = format!("{}/{}", &system_tmp_dir, &archive_name);
        compress_file(&_abs_archive_path, &abs_file_or_dirname)?;
        Ok((_abs_archive_path, archive_name))
    }
}

pub fn compress_file(archive_name: &String, absfilename: &String) -> Result<(), std::io::Error> {
    let mut input_buf = BufReader::new(File::open(absfilename)?);
    let archive_handle = File::create(archive_name)?;

    let mut encoder = GzEncoder::new(archive_handle, Compression::default());
    io::copy(&mut input_buf, &mut encoder)?;
    let _archive_handle = encoder.finish()?;

    Ok(())
}

pub fn compress_dir_shell(
    archive_name: &String,
    absdirname: &String,
) -> Result<(), std::io::Error> {
    if cfg!(target_os = "windows") {
        Command::new("tar")
            .arg("-zcf")
            .arg(archive_name)
            .arg(absdirname)
            .output()?
    } else {
        Command::new("tar")
            .arg("-zcf")
            .arg(archive_name)
            .arg(absdirname)
            .output()?
    };
    Ok(())
}

fn _compress_dir(
    archive_name: &String,
    absdirname: &String,
    basefilename: &String,
) -> Result<(), std::io::Error> {
    // let absdirname = absdirname.clone();
    let tar_gz_file_handle = File::create(archive_name)?;

    let encoder = GzEncoder::new(tar_gz_file_handle, Compression::default());
    let mut tar = tar::Builder::new(encoder);

    debug!("CREATING TAR {} ::: {}", basefilename, absdirname);
    tar.append_dir_all(basefilename, absdirname)?;
    debug!("Wrapping up TAR");
    tar.finish()?;
    debug!("TAR Done");
    Ok(())
}
