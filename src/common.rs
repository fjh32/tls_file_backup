
use core::arch;
use std::sync::Arc;
use env_logger::Env;
use log::{debug, error, log_enabled, info, Level};
use std::io::{self, BufReader};
use regex::Regex;
use std::fs::{metadata, File};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar;

pub fn get_secret_string(filename: &str) -> Arc<String> { 
    //arc is just a smart pointer that doesnt drop out of scope while any thread is still accessing it
    // eventually make this read from local file on runtime
    // let secret_string = Arc::new(Mutex::new(String::from("coconut"))); 
    // don't need mutex here because Read Only string, just need smart pointer across threads
    Arc::new(String::from("coconut"))
}

pub fn setup_logger() {
    env_logger::Builder::from_env(Env::default().filter_or("MY_LOG_LEVEL", "trace")).init();
}

pub fn make_address_str(addr: &String, port: &i32) -> String {
    let a = String::clone(addr);
    String::from(a + ":" + &port.to_string())
}

/// Client needs to send filename in form: "filename:example_filename.zip:filename"
pub fn verify_filename(filename:String) -> Result<String, io::Error> {
    let re = Regex::new(r"filename:([a-zA-Z0-9\._-]+):filename").unwrap();

    if let Some(mat) = re.find(&filename) {
        Ok(String::from(mat.as_str()))
    }
    else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid message"))
    }

    
}

/// creates archive if pathname is a dir, returns plain filename and absolute path filename
pub fn compress_file_for_send(pathname: String) -> Result<(), io::Error> {
    let filedata = metadata(pathname).unwrap();
    Ok(())
}

fn compress_file(filename: String, absfilename: String) -> Result<String, std::io::Error> {
    let mut input_buf = BufReader::new(File::open(absfilename)?);
    let archive_name = filename + ".zip";
    let archive_handle = File::create(&archive_name)?;
    let mut encoder = GzEncoder::new(archive_handle, Compression::default());
    io::copy(&mut input_buf, &mut encoder)?;
    let archive_handle = encoder.finish()?;
    Ok(archive_name)
}

fn compress_dir(dirname: String, absdirname: String) -> Result<String, std::io::Error> {
    let absdirname = absdirname.as_str();
    let archive_name = absdirname.to_string() + ".tar.gz";
    let tar_gz = File::create(&archive_name)?;
    let encoder = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    tar.append_dir_all(absdirname, absdirname)?;
    Ok(archive_name)
}