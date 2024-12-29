//! File system in os
mod inode;
mod stdio;

use crate::mm::UserBuffer;
/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
    /// Get file state
    fn get_fstate(&self, buf: *mut Stat);
}

/// The stat of a inode
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    pub dev: u64,
    pub ino: u64,
    pub mode: StatMode,
    pub nlink: u32,
    pad: [u64; 7],
}

bitflags! {
    /// The mode of a inode
    /// whether a directory or a file
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

pub use inode::{link_at, list_apps, open_file, unlink_at, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};
