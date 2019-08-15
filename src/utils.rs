extern crate nix;

use nix::sys::termios;
use std::fmt;
use std::fs::File;
use std::io::{stdin, stdout, Error, ErrorKind, Read, Write};
use std::os::unix::io::AsRawFd;
use std::thread::sleep;
use std::time::Duration;

// MAX_PASSPHRASE_LEN is required to preallocate
// String such that it will never be copied.
// https://github.com/zfsonlinux/zfs/blob/master/lib/libzfs/libzfs_crypto.c#L60.
const MAX_PASSPHRASE_LEN: usize = 512; // Does *not* include null-terminator.

pub fn getpass(prompt: &str) -> Result<String, Box<std::error::Error>> {
    // Avoid reallocations so we can zero out reliably.
    let mut s = String::with_capacity(MAX_PASSPHRASE_LEN + 1);

    // Adopted from the description in:
    // http://man7.org/linux/man-pages/man3/getpass.3.html.
    let tty = File::open("/dev/tty")?;
    let old_term = termios::tcgetattr(tty.as_raw_fd())?;
    let mut new_term = old_term.clone();
    new_term.local_flags &= !(termios::LocalFlags::ECHO | termios::LocalFlags::ISIG);
    new_term.local_flags |= termios::LocalFlags::ECHONL;
    termios::tcsetattr(tty.as_raw_fd(), termios::SetArg::TCSAFLUSH, &new_term)?;

    print!("{}", prompt);
    stdout().flush()?;
    stdin().read_line(&mut s)?;
    if s.ends_with('\n') {
        s.pop();
    }

    // Restore flags.
    termios::tcsetattr(tty.as_raw_fd(), termios::SetArg::TCSAFLUSH, &old_term)?;

    Ok(s)
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
struct PollPassphraseTimeoutError;

impl fmt::Display for PollPassphraseTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "Timeout waiting for poll".to_owned().fmt(f)
    }
}

impl std::error::Error for PollPassphraseTimeoutError {}

pub fn poll_passphrase_ready() -> Result<(), Box<std::error::Error>> {
    let mut buf = String::new();
    for i in 0..50 {
        let mut vcs = File::open("/dev/vcs1")?;
        vcs.read_to_string(&mut buf)?;

        if buf.to_lowercase().contains("passphrase") {
            return Ok(());
        }

        buf.truncate(0);
        sleep(Duration::from_millis((100 * 2u64.pow(i)).min(1000)));
    }

    Err(Box::new(PollPassphraseTimeoutError))
}

// -----------------------------------------------------------------------------

pub fn dump_console() -> Result<(), Box<std::error::Error>> {
    // Emulate setterm -dump 1 -file /dev/stdout.
    // vcsa is described in http://man7.org/linux/man-pages/man4/vcs.4.html.
    let vcs = File::open("/dev/vcsa1")?;
    let mut header = vec![];
    let mut handle = vcs.take(4);
    handle.read_to_end(&mut header)?;

    let rows = header[0] as usize;
    let cols = header[1] as usize;
    if rows * cols == 0 {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidData,
            "cannot read /dev/vcsa1",
        )));
    }

    let mut handle = handle.into_inner();
    let mut inbuf = String::with_capacity(rows * cols * 2);
    let sz = handle.read_to_string(&mut inbuf)?;
    if sz != rows * cols * 2 {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidData,
            "cannot read /dev/vcsa1",
        )));
    }

    let mut outbuf = String::with_capacity(rows * cols);
    for (i, c) in inbuf.chars().step_by(2).into_iter().enumerate() {
        outbuf.push(c);
        if (i + 1) % cols == 0 {
            while outbuf.ends_with(' ') {
                outbuf.pop();
            }

            if i + 1 != rows * cols {
                outbuf.push('\n');
            }
        }
    }

    stdout().write_all(outbuf.as_bytes())?;

    Ok(())
}
