extern crate nix;

use nix::sys::termios;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::os::unix::io::AsRawFd;

// MAX_PASSPHRASE_LEN is required to preallocate
// String such that it will never be copied.
// https://github.com/zfsonlinux/zfs/blob/master/lib/libzfs/libzfs_crypto.c#L60.
const MAX_PASSPHRASE_LEN: usize = 512; // Does *not* include null-terminator.

pub fn getpass() -> Result<String, Box<std::error::Error>> {
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

    print!("Enter your passphrase: ");
    stdout().flush()?;
    stdin().read_line(&mut s)?;
    if s.ends_with('\n') {
        s.pop();
    }

    // Restore flags.
    termios::tcsetattr(tty.as_raw_fd(), termios::SetArg::TCSAFLUSH, &old_term)?;

    Ok(s)
}
