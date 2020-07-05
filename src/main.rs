//! This program kills Microsoft Word on a Mac. It is useful since MsWord often
//! fails to die (because of updates) and this blocks computer shutdown.
//! Hence, having a simple program to automatically kill the whole stuff might 
//! make my wife's life easier.
//! 
//! # Note:
//! This program really could've been programmed with bash. However, it is easier
//! for her to just click on a simple application than having to spawn a shell and
//! remember the appropriate way to launch the program. And it is easier for me to
//! just hand her a binary iso having to set the executable flag of some text script
//! on her machine. This way, it just works.
//! 
//! Author: X. Gillard
//! Date  : 24-10-2018.
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::str;
use std::ops::Add;
use std::process::{Command, Output};
use regex::Regex;

lazy_static! {
static ref PSLINE:String = String::from(r"(?P<UID>\d+)").add(r"\s+")
                    .add(r"(?P<PID>\d+)").add(r"\s+")
                    .add(r"(?P<PPID>\d+)").add(r"\s+")
                    .add(r"(?P<REST>(\d|\w|\s)+)").add(r"\s+");

static ref RPSLINE:Regex = Regex::new(&PSLINE).unwrap();
}

/// My crate's errors
#[derive(Debug)]
enum Error {
    Text       {e: String},
    Conversion {e: std::str::Utf8Error},
    IO         {e: std::io::Error}
}
impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Text {e:s.to_string()}
    }
}
impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Text {e:s}
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Error {
        Error::Conversion {e}
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO {e}
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Returns the pid from the given `psline`. Where `psline` is a line 
/// extracted from  a call to `ps alx`. When no such information could
/// be found, None is returned instead.
fn extract_pid<'a>(psline: &'a str) -> Result<&'a str> {
    match RPSLINE.captures(psline) {
        Some(ref captures) => Ok(captures.name("PID").unwrap().as_str()),
        None               => Err("No match in `psline`".into())
    }
}

/// Sends the SIGKILL signal to the given `process`.
fn kill(process: &str) -> Result<Output> {
    Ok (
        Command::new("kill")
        .args(&["-9", process])
        .output()?
    )
}

/// Best effort attempt to send the SIGKILL signal the the process outlined
/// by the given `psline`. `psline` is a line extracted from the output of 
/// a call to `ps alx`.
fn do_kill(psline: &str) -> Result<Output> {
    kill( &extract_pid(psline)? )
}

/// Lists all the lines of a `ps alx` which contain the given string
/// 'Microsoft Word'
fn pslines() -> Result<Vec<String>> {
    let ps = Command::new("ps")
        .arg("alx")
        .output()?;

    if !ps.status.success() {
        Err("`ps alx` failed for some reason".into())
    } else {
        let text   = str::from_utf8(&ps.stdout)?;
        let lines  = text.lines()
            .filter(|line| line.contains("Microsoft"))
            .map(String::from)
            .collect::<Vec<String>>();
        Ok(lines)
    }
}

/// This is the program's entry point.
fn main() -> Result<()> {
    for line in pslines()? {
        println!("Killing {}", extract_pid(&line)?);
        do_kill(&line)?;
    }

    Ok(())
}
