extern crate libc;
extern crate nix;

mod getpass;

use getpass::getpass;
use std::ffi::CString;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::ptr::write_volatile;

// MAX_PASSPHRASE_LEN is required to preallocate
// String such that it will never be copied.
// https://github.com/zfsonlinux/zfs/blob/master/lib/libzfs/libzfs_crypto.c#L60.
const MAX_PASSPHRASE_LEN: usize = 512; // Does *not* include null-terminator.

nix::ioctl_write_ptr_bad!(tiocsti, libc::TIOCSTI, libc::c_char);

fn main() -> Result<(), Box<std::error::Error>> {
    // Avoid reallocations so we can zero out reliably.
    let mut passphrase = String::with_capacity(MAX_PASSPHRASE_LEN + 1);
    getpass(&mut passphrase)?;
    let passphrase = CString::new(passphrase)?;

    // Now, write passphrase + newline into /dev/console.
    {
        let console = File::open("/dev/console")?;
        for i in 0..passphrase.as_bytes().len() {
            unsafe {
                tiocsti(console.as_raw_fd(), passphrase.as_ptr().offset(i as isize))?;
            }
        }
        unsafe {
            tiocsti(console.as_raw_fd(), &('\n' as libc::c_char))?;
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
