
use std::fs;
use std::path::PathBuf;
use std::path::Path;

use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use chrono::serde::ts_microseconds;
use std::io;
use std::io::ErrorKind::InvalidData;

use std::process::{Command, Stdio};
use std::fmt::Display;

mod config;
use crate::config::*;

const SSH: &str = "/usr/bin/ssh";
const SCP: &str = "/usr/bin/scp";

#[derive(PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
struct Timestamp {
    #[serde(with = "ts_microseconds")]
    timepoint: DateTime<Utc>
}

impl Timestamp {

  fn new() -> Timestamp {
      Timestamp { timepoint: DateTime::<Utc>::MIN_UTC }
  }

  // Return a copy of the time variable wrapped/encapsulated by `Timestamp'
  fn get(&self) -> DateTime<Utc> {
      self.timepoint.clone()
  }

  // Update the timestamp to the current time
  fn bump(&mut self) {
      self.timepoint = Utc::now();
  }

  fn read() -> Option<Self> {
    let timestamp_data = std::fs::read_to_string(CACHE_FILE).ok()?;
    serde_yaml::from_str(&timestamp_data).ok()
  }

  fn commit(self) -> Result<Self, io::Error> {
      // Try to write the timestamp
      // If we can't, log or print an error and quit with a negative return code
      let timestamp_data = serde_yaml::to_string(&self)
                            .expect("Error serializing timestamp data");
      std::fs::write(CACHE_FILE, &timestamp_data)?;
      Ok(self)
  }
}

// Upload a file using `scp'
fn scp_upload(source: String, dest: String, options: &Vec<String>) -> Result<(), String> {
    let mut args = options.clone();
    args.extend([source.into(), dest.into()]);

    let mut child = Command::new(SCP);
    child.args(&args);

    let status = child.status();
    match status {
        Err(s) => Err(format!("Error running `{} {}': {}", SCP, args.join(" "), s)),
        Ok(s) => match s.success() {
            true => Ok(()),
            false => Err(format!("`{} {}' exited with non-zero status", SCP, args.join(" ")))
        }
    }
}

fn ssh_exec(account: &str, cmd: String, options: &Vec<String>) -> Result<(), String> {
    let mut args = options.clone();
    args.extend([account.into(), cmd.into()]); // into() is now optional for cmd

    let mut child = Command::new(SSH);
    child.args(&args);

    let status = child.status();
    match status {
        Err(s) => Err(format!("Error running `{} {}': {}", SSH, args.join(" "), s)),
        Ok(s) => match s.success() {
            true => Ok(()),
            false => Err(format!("`{} {}' exited with non-zero status", SSH, args.join(" ")))
        }
    }
}

// Given a path (which may or may not be a symlink), find the path of the corresponding regular file
fn resolve_link(path: String) -> Result<String, io::Error> {
    let mut path_buf = PathBuf::new();
    let mut path = Path::new(&path);

    if path.is_symlink() {
        path_buf = path.canonicalize()?;
        path = &path_buf;
    }
    Ok(path.to_str().ok_or(InvalidData)?.to_string())
}

// Determine the time of the local certificates
fn cert_timestamp(live_cert_dir: &str) -> Result<DateTime<Utc>, io::Error> {
    let mut newest: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;
    let mut num_files = 0;

    for entry in fs::read_dir(live_cert_dir)? {
        let entry = entry?;
        let mut metadata = entry.metadata()?;

        // Compiler error occurs when these two are a single statement:
        let path = entry.path();
        let filename = path.file_name().unwrap_or_default().to_str().unwrap_or("");

        if (metadata.is_file() || metadata.is_symlink()) && filename.ends_with(".pem") {
            num_files += 1;

            if metadata.is_symlink() {
                metadata = fs::canonicalize(entry.path())?.metadata()?;
            }
            let modified = metadata.modified()?;
            let modified = chrono::DateTime::<Utc>::from(modified);

            if newest < modified {
                newest = modified;
            }
        }
    }

    if num_files == 0 {
        Err(InvalidData.into())
    }
    else {
        Ok(newest)
    }
}

fn main() {
    let cert_timestamp = cert_timestamp(&format!("{LIVE_CERT_DIR}"))
        .expect("Problem reading `live' certificate directory");

    let mut upload_timestamp = match Timestamp::read() {
        Some(t) => t,
        None => Timestamp::new(),
    };

    // Alternatively, .to_rfc3339() is an option
    println!("Local certificate updated:  {}", cert_timestamp);
    println!("Server certificate updated: {}", upload_timestamp.get());

    if cert_timestamp > upload_timestamp.get() {
        println!("Uploading certificate...");

        let mut scp_options = Vec::<String>::new();
        scp_options.extend(["-P".into(), SSH_PORT.into()]);
        scp_upload(resolve_link(format!("{LIVE_CERT_DIR}/fullchain.pem")).unwrap(),
                format!("{SERVER_SSH_ACCOUNT}:{SERVER_CERT_DIR}/public.crt"),
                &scp_options)
                .unwrap();
        scp_upload(resolve_link(format!("{LIVE_CERT_DIR}/privkey.pem")).unwrap(),
                format!("{SERVER_SSH_ACCOUNT}:{SERVER_CERT_DIR}/private.key"),
                &scp_options)
                .unwrap();

        let mut ssh_options = Vec::<String>::new();
        ssh_options.extend(["-p".into(), SSH_PORT.into()]);
        // Note: SSH passes back the exit status of the remote command we ran.
        // This means that, for example, /bin/false will cause an error like we want.
        ssh_exec(SERVER_SSH_ACCOUNT,
                format!("chown -R minio:minio {SERVER_CERT_DIR} && docker kill --signal HUP minio"),
                &ssh_options)
                .unwrap();

        upload_timestamp.bump();
        upload_timestamp.commit();
    } else {
        println!("Nothing to do: The server's certificates are up to date.");
    }
}


/*
Known Bugs
---------
The ssh remote command isn't escaped with "'" in error messages.
*/
