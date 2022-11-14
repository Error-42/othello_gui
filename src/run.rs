use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub fn run(path: OsString, input: &[u8]) -> io::Result<String> {
    let mut proc = if cfg!(target_os = "windows") {
        Command::new("cmd")
    } else {
        todo!("Implement running for linux")
    };

    let handle = if cfg!(target_os = "windows") {
        proc.arg("/C")
    } else {
        todo!("Implement running for linux")
    };

    let mut child = handle
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(input)?;

    // child.wait_with_output().map(|o| o.stdout)

    let mut output = String::new();

    child.stdout.unwrap().read_to_string(&mut output)?;

    Ok(output)
}

pub fn run_timeout(path: OsString, input: &[u8], timeout: Duration) -> io::Result<Option<String>> {
    let mut proc = if cfg!(target_os = "windows") {
        Command::new("cmd")
    } else {
        todo!("Implement running for linux")
    };

    let handle = if cfg!(target_os = "windows") {
        proc.arg("/C")
    } else {
        todo!("Implement running for linux")
    };

    let mut child = handle
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(input)?;

    match child.wait_timeout(timeout)? {
        None => Ok(None),
        Some(_) => {
            let mut output = String::new();
            child.stdout.unwrap().read_to_string(&mut output)?;
            Ok(Some(output))
        }
    }
}
