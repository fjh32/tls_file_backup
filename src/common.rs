use std::path::Path;
use std::process::Command;
use env_logger::Env;
use std::io::{self, BufReader};
use regex::Regex;
use std::fs::{metadata, File};
use flate2::write::GzEncoder;
use flate2::Compression;
use log::debug;
use tar;

pub fn setup_logger() {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "info")).init();
}

pub fn make_address_str(addr: &String, port: &i32) -> String {
    let a = String::clone(addr);
    String::from(a + ":" + &port.to_string())
}

/// Client needs to send filename in form: "filename:example_filename.zip:filename"
pub fn verify_filename(filename:String) -> Result<String, io::Error> {
    let re = Regex::new(r"filename:([a-zA-Z0-9\._-]+):filename").unwrap();

    if re.is_match(&filename) {
        let extracted_filename = filename.replace("filename:", "").replace(":filename", "");
        Ok(String::from(extracted_filename))
    }
    else {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid message"))
    }

    
}

/// compresses abs_file_or_dirname into ./file_or_dirname.gz if a file or ./file_or_dirname.tar.gz if a dir
pub fn compress(abs_file_or_dirname: String) -> Result<(String,String), std::io::Error> {
    let filedata = metadata(&abs_file_or_dirname)?;
    let path = Path::new(&abs_file_or_dirname);
    let basefilename = path.file_name().unwrap().to_str().unwrap();

    let archive_dir = path.parent().unwrap().to_str().unwrap();

    if filedata.is_dir() {
        let archive_name = format!("{}.tar.gz", basefilename);
        let _abs_archive_path = format!("{}/{}", &archive_dir, &archive_name);
        // compress_dir(&abs_archive_path, &abs_file_or_dirname, &basefilename.to_string())?;
        compress_dir_shell(&archive_name, &abs_file_or_dirname)?;
        Ok((archive_name.clone(), archive_name))
    }
    else {
        let archive_name = format!("{}.gz", basefilename);
        let _abs_archive_path = format!("{}/{}", &archive_dir, &archive_name);
        compress_file(&archive_name, &abs_file_or_dirname)?;
        Ok((archive_name.clone(), archive_name))
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

pub fn compress_dir_shell(archive_name: &String, absdirname: &String) -> Result<(), std::io::Error> {
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

fn _compress_dir(archive_name: &String, absdirname: &String, basefilename: &String) -> Result<(), std::io::Error> {
    // let absdirname = absdirname.clone();
    let tar_gz_file_handle = File::create(archive_name)?;

    let encoder = GzEncoder::new(tar_gz_file_handle, Compression::default());
    let mut tar = tar::Builder::new(encoder);

    debug!("CREATING TAR {} ::: {}", basefilename, absdirname);
    tar.append_dir_all(&basefilename, absdirname)?;
    debug!("Wrapping up TAR");
    tar.finish()?;
    debug!("TAR Done");
    Ok(())
}