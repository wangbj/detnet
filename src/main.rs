#![allow(unused_imports)]
#![allow(dead_code)]

extern crate nix;
extern crate combine;
extern crate libc;

use std::thread;
use std::fs;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use std::os::unix::net::{UnixStream, UnixListener};
use nix::unistd::*;
use libc::{uid_t, gid_t};
use detnet;

fn handle_client(mut stream: UnixStream) {
    let mut message = String::new();
    stream.read_to_string(&mut message).unwrap();
    println!("{}", message);
}

fn check_user_root() -> std::io::Result<()> {
    unsafe { let euid = libc::geteuid() as u32;
             match euid {
                 0 => Ok(()),
                 _ => Err(Error::new(ErrorKind::PermissionDenied, "must run as root")),
             }
    }
}

fn detnet_main() {
    check_user_root().unwrap();
    let dettrace_group = detnet::from_group("dettrace").expect("dettrace group not found");
    let unp = "/var/run/dettrace.sock";
    fs::remove_file(unp).ok();
    let listener = UnixListener::bind(unp).unwrap();
    let mut perms = fs::metadata(unp).unwrap().permissions();
    perms.set_mode(0o660);
    fs::set_permissions(unp, perms).expect(&format!("change {} permission to 0660", unp));

    let uid = nix::unistd::Uid::from_raw(0);
    let gid = nix::unistd::Gid::from_raw(dettrace_group as gid_t);
    nix::unistd::chown(unp, Some(uid), Some(gid)).expect(&format!("failed to chown {}", unp));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(||handle_client(stream));
            }
            Err(_err) => {
                break;
            }
        }
    }
}

fn main() {
    detnet_main();
}

#[cfg(test)]
mod tests {
    #[test]
    fn root_group_exists() {
        assert_eq!(detnet::from_group("root").unwrap_or(65535), 0);

    }
    #[test]
    fn nonexist_group() {
        assert_eq!(detnet::from_group("doesnotExist").unwrap_or(65535), 65535);
    }
}
