extern crate libc;
extern crate nix;

use std::ffi;
use std::fs;
use std::io;
use std::io::Write;
use std::os::unix::{io::AsRawFd};
use std::ptr::write_volatile;
use std::string;

// MAX_PASSPHRASE_LEN is required to preallocate
// String such that it will never be copied.
// https://github.com/zfsonlinux/zfs/blob/master/lib/libzfs/libzfs_crypto.c#L60.
const MAX_PASSPHRASE_LEN: usize = 512; // Does *not* include null-terminator.

nix::ioctl_write_ptr_bad!(tiocsti, libc::TIOCSTI, libc::c_char);

fn main() -> Result<(), Box<std::error::Error>> {
    // Adopted from the description in:
    // http://man7.org/linux/man-pages/man3/getpass.3.html.
    let tty = fs::OpenOptions::new()
        .read(true)
        .open("/dev/tty")?;
    let oflags = nix::sys::termios::tcgetattr(tty.as_raw_fd())?;
    let mut nflags = oflags.clone();
    nflags.local_flags &= !nix::sys::termios::LocalFlags::ECHO;
    nflags.local_flags |= nix::sys::termios::LocalFlags::ECHONL;
    nix::sys::termios::tcsetattr(tty.as_raw_fd(), nix::sys::termios::SetArg::TCSANOW, &nflags)?;

    // Get the passphrase
    print!("Enter your passphrase: ");
    io::stdout().flush()?;
    // Avoid reallocations so we can zero out reliably.
    let mut passphrase = string::String::with_capacity(MAX_PASSPHRASE_LEN + 1);
    io::stdin().read_line(&mut passphrase)?;
    let passphrase = ffi::CString::new(passphrase)?;

    // Restore echo.
    nix::sys::termios::tcsetattr(tty.as_raw_fd(), nix::sys::termios::SetArg::TCSANOW, &oflags)?;

    // Now, write passphrase into /dev/console.
    let console = fs::OpenOptions::new()
        .read(true)
        .open("/dev/console")?;

    for i in 0..passphrase.as_bytes().len() {
        unsafe {
            tiocsti(console.as_raw_fd(), passphrase.as_ptr().offset(i as isize))?;
        }
    }

    // Zero out passphrase from memory.
    // Kind of pointless because the real
    // symmetric key is unwrapped and kept in
    // memory. Oh well.
    for mut elem in passphrase.into_bytes() {
        unsafe {
            write_volatile(&mut elem, 0);
        }
    }

    Ok(())
}
