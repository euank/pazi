use termion::screen::AlternateScreen;
use std::io::{Stdin, Stdout, Write};
use termion::{clear, cursor};
use std::io::Error as IOErr;
use std::fmt;
use std::convert::From;

pub fn filter(
    opts: Vec<(&String, f64)>,
    stdin: Stdin,
    stdout: Stdout,
) -> Result<&String, FilterError> {
    let mut alt = AlternateScreen::from(stdout);
    write!(alt, "{}{}", clear::All, cursor::Goto(1, 1))?;
    for i in 0..opts.len() {
        write!(alt, "{}\t{}\t{}\n", i, opts[i].1, opts[i].0)?;
    }
    write!(alt, "> ")?;
    alt.flush()?;
    let mut input = String::new();
    stdin
        .read_line(&mut input)
        .map_err(|err| format!("could not read stdin: {}", err))?;
    let chosen = input
        .trim()
        .parse::<usize>()
        .map_err(|err| format!("could not parse input: {}", err))?;

    opts.get(chosen)
        .map(|e| e.0)
        .ok_or(FilterError::String("index out of bounds".to_string()))
}

#[derive(Debug)]
pub enum FilterError {
    WriteErr(IOErr),
    String(String),
}

impl From<IOErr> for FilterError {
    fn from(e: IOErr) -> Self {
        FilterError::WriteErr(e)
    }
}

impl From<String> for FilterError {
    fn from(e: String) -> Self {
        FilterError::String(e)
    }
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &FilterError::String(ref s) => s.fmt(f),
            &FilterError::WriteErr(ref ioe) => ioe.fmt(f),
        }
    }
}
