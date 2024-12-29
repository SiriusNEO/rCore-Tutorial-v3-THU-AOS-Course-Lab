use crate::{
    mm::{MapPermission, VirtAddr},
    task,
};

const VA_MAX: usize = usize::MAX;

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    // 1. Align to page size (4k)
    // 2, 3. Illegal prot
    // 4. No space
    if start & 0xfff != 0 || prot & !0x7 != 0 || prot & 0x7 == 0 || VA_MAX - len <= start {
        return -1;
    }

    // let mut map_perm = MapPermission::U;
    // if prot & 0x1 != 0 {
    //     map_perm |= MapPermission::R;
    // }
    // if prot & 0x2 != 0 {
    //     map_perm |= MapPermission::W;
    // }
    // if prot & 0x4 != 0 {
    //     map_perm |= MapPermission::X;
    // }
    let map_perm = MapPermission::from_bits_truncate((prot << 1) as u8) | MapPermission::U; // a fast way
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    task::map_in_current_task(start_vpn, end_vpn, map_perm)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    // 1. Align to page size (4k)
    // 2. No space
    if start & 0xfff != 0 || VA_MAX - len <= start {
        return -1;
    }

    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    task::unmap_in_current_task(start_vpn, end_vpn)
}
