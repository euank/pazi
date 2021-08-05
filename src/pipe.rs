use std::convert::From;
use std::fmt;
use std::io::prelude::*;
use std::io::Error as IOErr;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use log::debug;

pub fn pipe<I>(opts_iter: I, pipe: Vec<&str>) -> Result<String>
where
    I: Iterator<Item = (String, f64)>,
{
    let mut pipe = pipe.iter();
    let opts = opts_iter.collect::<Vec<_>>();

    let program = match pipe.next() {
        None | Some(&"") => {
            bail!("invalid pipe program: empty");
        }
        Some(s) => s,
    };

    let mut cmd = Command::new(program);
    cmd.args(pipe.collect::<Vec<_>>());
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut process = match cmd.spawn() {
        Err(e) => {
            bail!("couldn't spawn pipe process {}: {}", program, e);
        }
        Ok(process) => process,
    };

    let take_stdin = process.stdin.take();
    let mut stdin = take_stdin.unwrap();
    let mut input_lines = Vec::new();
    for opt in &opts {
        let line = format!("{}\t{}", opt.1, opt.0);
        input_lines.push(line.clone());
        match writeln!(stdin, "{}", line) {
            Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                break;
            }
            e => e.with_context(|| "error writing input line to pipe program")?,
        }
    }
    std::mem::drop(stdin);

    process.wait()?;
    let mut s = String::new();
    process.stdout.unwrap().read_to_string(&mut s)?;
    let line = match s.split('\n').next() {
        None => {
            bail!("pipe program did not produce any output lines".to_string());
        }
        Some(line) => line,
    };
    // find the input line for this output
    for (ndx, orig_line) in input_lines.iter().enumerate() {
        debug!("{} == {}", orig_line, line);
        if orig_line == line {
            // Intentionally return the `opts` version of it since we may end up mutating things
            // for display soon.
            return Ok(opts[ndx].0.to_string());
        }
    }
    bail!("pipe program did not produce a line from its input");
}

#[derive(Debug)]
pub enum PipeError {
    WriteErr(IOErr),
    String(String),
}

impl From<IOErr> for PipeError {
    fn from(e: IOErr) -> Self {
        PipeError::WriteErr(e)
    }
}

impl From<String> for PipeError {
    fn from(e: String) -> Self {
        PipeError::String(e)
    }
}

impl fmt::Display for PipeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipeError::String(ref s) => s.fmt(f),
            PipeError::WriteErr(ref ioe) => ioe.fmt(f),
        }
    }
}
