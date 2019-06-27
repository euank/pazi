use crate::channel;
use signal_hook;
use std::convert::From;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::io::Error as IOErr;
use std::io::{Stdin, Write};
use std::process::{Command, Stdio};
use std::thread;
use termion::screen::AlternateScreen;
use termion::{clear, cursor};

pub fn filter<I>(opts_iter: I, stdin: Stdin, stdout: fs::File) -> Result<String, FilterError>
where
    I: Iterator<Item = (String, f64)>,
{
    // if either aren't a tty, we can't really do an interactive selection, just print stuff
    // out.
    let istty = unsafe {
        let stdout_tty = libc::isatty(libc::STDOUT_FILENO);
        let stdin_tty = libc::isatty(libc::STDIN_FILENO);
        stdout_tty != 0 && stdin_tty != 0
    };
    if !istty {
        return Err(FilterError::NoSelection);
    }

    // So, this is a massive abstraction leak, but unix signals are icky so it's not really
    // surprising.
    // Because we're popping over to an alternative screen buffer, we need to restore the teriminal
    // when we're done, even if we get sigint / sigterm.
    // This mess is to handle that.
    // Basically, to restore the screen buffer we just need to return from this fn so
    // 'alt' is dropped.
    // We do this by waiting for signals or user-input, and just returning on whichever we see
    // first.
    // That makes 'sigint'/'sigterm' result in us exiting.
    let mut alt = AlternateScreen::from(stdout);
    let signal = notify(&[signal_hook::SIGINT, signal_hook::SIGTERM])
        .map_err(|err| format!("error setting sigint hook: {}", err))?;
    let (suser_input, ruser_input) = channel::bounded(0);
    // Wait for a signal or for the user to select a choice.
    write!(alt, "{}{}", clear::All, cursor::Goto(1, 1))?;

    let opts = {
        let mut opts = opts_iter.collect::<Vec<_>>();
        opts.reverse();
        opts
    };

    for i in 0..opts.len() {
        write!(alt, "{}\t{}\t{}\n", opts.len() - i, opts[i].1, opts[i].0)?;
    }
    write!(alt, "> ")?;
    alt.flush()?;
    thread::spawn(move || {
        // Since threads can be messy, do the minimum possible in the thread.
        let mut input = String::new();
        // ignore result, we're intentionally racing input and ctrl-c signals
        let _ = suser_input.send(stdin.read_line(&mut input).map(|_| input));
    });
    select! {
        recv(signal) -> _ => {
            return Err(FilterError::NoSelection);
        },
        recv(ruser_input) -> res => {
            return res.unwrap()
                .map_err(|_| FilterError::String("unable to read input".to_string()))
                .and_then(|val| {
                    if val.trim().is_empty() {
                        debug!("user input is empty");
                        Err(FilterError::NoSelection)
                    } else {
                        Ok(val)
                    }
                })
                .and_then(|val| {
                    val.trim().parse::<usize>()
                        .map_err(|e| {
                            FilterError::String(format!("unable to parse selection: {}", e))
                        })
                })
                .and_then(|ndx| {
                    if ndx > opts.len() {
                        Err(FilterError::String("index out of bounds".to_string()))
                    } else if ndx > opts.len() {
                        // handle separately from 'opts.get' to avoid underflow panicing
                        Err(FilterError::String("index out of bounds".to_string()))
                    } else {
                        opts.get(opts.len() - ndx).map(|o| o.0.clone())
                            .ok_or(FilterError::String("index out of bounds".to_string()))
                    }
                });
        },
    };
}

pub fn pipe<I>(opts_iter: I, pipe: Vec<&str>) -> Result<String, PipeError>
where
    I: Iterator<Item = (String, f64)>,
{
    let mut pipe = pipe.iter();
    let opts = opts_iter.collect::<Vec<_>>();

    let program = match pipe.next() {
        None => {
            return Err(PipeError::String("invalid pipe program: empty".to_string()));
        }
        Some(&"") => {
            return Err(PipeError::String("invalid pipe program: empty".to_string()));
        }
        Some(s) => s,
    };

    let mut cmd = Command::new(program);
    cmd.args(pipe.collect::<Vec<_>>());
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut process = match cmd.spawn() {
        Err(e) => panic!("couldn't spawn pipe process {}: {}", program, e),
        Ok(process) => process,
    };

    let take_stdin = process.stdin.take();
    let mut stdin = take_stdin.unwrap();
    for i in 0..opts.len() {
        match write!(stdin, "{}\t{}\t{}\n", opts.len() - i, opts[i].1, opts[i].0) {
            Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                break;
            }
            e => e?,
        }
    }
    std::mem::drop(stdin);

    process.wait()?;
    let mut s = String::new();
    process.stdout.unwrap().read_to_string(&mut s)?;
    // assume the selected item is of the same format we printed
    let s = match s.splitn(3, "\t").nth(2) {
        None => {
            return Err(PipeError::String(
                "pipe program did not produce a line from its input".to_string(),
            ));
        }
        Some(s) => s,
    };

    Ok(s.to_string())
}

fn notify(signals: &[i32]) -> Result<channel::Receiver<i32>, IOErr> {
    let (s, r) = channel::bounded(100);
    let signals = signal_hook::iterator::Signals::new(signals)?;
    thread::spawn(move || {
        for signal in signals.forever() {
            // ignore result, we're intentionally racing input and ctrl-c signals
            let _ = s.send(signal);
        }
    });
    Ok(r)
}

#[derive(Debug)]
pub enum FilterError {
    WriteErr(IOErr),
    String(String),
    NoSelection,
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
            &FilterError::NoSelection => Ok(()),
            &FilterError::WriteErr(ref ioe) => ioe.fmt(f),
        }
    }
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
            &PipeError::String(ref s) => s.fmt(f),
            &PipeError::WriteErr(ref ioe) => ioe.fmt(f),
        }
    }
}
