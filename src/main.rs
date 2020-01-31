extern crate libc;
extern crate nix;

mod utils;

use std::ffi::CString;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::ptr::write_volatile;
use utils::{dump_console, getpass, poll_passphrase_ready};

nix::ioctl_write_ptr_bad!(tiocsti, libc::TIOCSTI, libc::c_char);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    poll_passphrase_ready()?;
    dump_console()?;

    // Write passphrase + newline into /dev/console.
    let passphrase = CString::new(getpass("")?)?;
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

    // TODO: Give indication passphrase succeeded/failed
    // or continually retry until Dracut kills sshd.

    Ok(())
}
