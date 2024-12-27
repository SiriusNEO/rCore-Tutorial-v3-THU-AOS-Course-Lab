const FD_STDOUT: usize = 1;

use crate::batch::{get_app_addr_range, get_user_stack_addr_range};

fn check(range: (usize, usize), value: (usize, usize)) -> bool {
    (range.0 <= value.0 && value.0 < range.1) && (range.0 <= value.1 && value.1 < range.1)
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let app_addr_range = get_app_addr_range();
    let us_addr_range = get_user_stack_addr_range();
    let buf_addr = (buf as usize, buf as usize + len);
    // println!(
    //     "app_range: [{:#x}, {:#x})\n",
    //     app_addr_range.0, app_addr_range.1
    // );
    // println!("buf_addr: [{:#x}, {:#x})\n", buf_addr.0, buf_addr.1);
    if !check(app_addr_range, buf_addr) && !check(us_addr_range, buf_addr) {
        println!("Error: sys write out of address range");
        return -1 as isize;
    }

    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            // panic!("Unsupported fd in sys_write!");
            return -1 as isize;
        }
    }
}
