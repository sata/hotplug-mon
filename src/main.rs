// https://github.com/Smithay/udev-rs/blob/master/examples/monitor.rs
// adapted only how socket is setup to listen to different events

use libc;
use std::os::unix::io::AsRawFd;
use udev;

use std::io;
use std::ptr;
use std::thread;
use std::time::Duration;

use libc::{c_int, c_short, c_ulong, c_void};

#[repr(C)]
struct pollfd {
    fd: c_int,
    events: c_short,
    revents: c_short,
}

#[repr(C)]
struct sigset_t {
    __private: c_void,
}

#[allow(non_camel_case_types)]
type nfds_t = c_ulong;

const POLLIN: c_short = 0x0001;

extern "C" {
    fn ppoll(
        fds: *mut pollfd,
        nfds: nfds_t,
        timeout_ts: *mut libc::timespec,
        sigmask: *const sigset_t,
    ) -> c_int;
}

fn main() -> io::Result<()> {
    let mut sock = match setup_socket() {
        Ok(sock) => sock,
        Err(_) => panic!("error"),
    };

    let mut fds = vec![pollfd {
        fd: sock.as_raw_fd(),
        events: POLLIN,
        revents: 0,
    }];

    loop {
        let result = unsafe {
            ppoll(
                (&mut fds[..]).as_mut_ptr(),
                fds.len() as nfds_t,
                ptr::null_mut(),
                ptr::null(),
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        let event = match sock.next() {
            Some(evt) => evt,
            None => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        println!(
            "{}: {} {} (subsystem={}, sysname={}, devtype={})",
            event.sequence_number(),
            event.event_type(),
            event.syspath().to_str().unwrap_or("---"),
            event
                .subsystem()
                .map_or("", |s| { s.to_str().unwrap_or("") }),
            event.sysname().to_str().unwrap_or(""),
            event.devtype().map_or("", |s| { s.to_str().unwrap_or("") })
        );
    }
}

fn setup_socket() -> std::io::Result<udev::MonitorSocket> {
    let sock = udev::MonitorBuilder::new()?
        .match_subsystem("drm")?
        .listen()?;

    Ok(sock)
}
