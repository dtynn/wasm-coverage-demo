#![feature(no_coverage)]

#[link(wasm_import_module = "my_profile")]
extern "C" {
    #[link_name = "my_output_profile"]
    fn my_output_profile(bytes_ptr: *const u8, bytes_len: u32) -> i32;
}

#[no_mangle]
pub extern "C" fn compare(left: i32, right: i32) -> i32 {
    if left > right {
        return 1;
    }

    if left < right {
        return -1;
    }

    0
}

#[no_mangle]
pub extern "C" fn compare_rev(left: i32, right: i32) -> i32 {
    return compare(right, left);
}

#[no_mangle]
#[no_coverage]
pub extern "C" fn reset_cov() {
    minicov::reset_coverage();
}

#[no_mangle]
#[no_coverage]
pub extern "C" fn output_cov() -> i32 {
    const BUF_SIZE: usize = 4096;
    let size = minicov::get_coverage_data_size();
    if size > BUF_SIZE {
        return -1;
    }

    let mut buf = [0u8; BUF_SIZE];
    minicov::capture_coverage_to_buffer(&mut buf[..size]);

    let ret = unsafe { my_output_profile(buf.as_ptr(), size as u32) };
    if ret == -1 {
        return -3;
    }

    0
}
