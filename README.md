# wasm-coverage-demo
This demo shows how to get profile info for wasm from inside the wasm runtime.

## usage

### prepare
```
./install-dep.sh
```

this ensures that the `wasm32-unknown-unknown` target & llvm-tools-preview are installed.

### run
```
./show-cover.sh 3 5
```

this script will output a human-readable coverage report like

```
    1|       |#![feature(no_coverage)]
    2|       |
    3|       |#[link(wasm_import_module = "my_profile")]
    4|       |extern "C" {
    5|       |    #[link_name = "my_output_profile"]
    6|       |    fn my_output_profile(bytes_ptr: *const u8, bytes_len: u32) -> i32;
    7|       |}
    8|       |
    9|       |#[no_mangle]
   10|      1|pub extern "C" fn compare(left: i32, right: i32) -> i32 {
   11|      1|    if left > right {
   12|      0|        return 1;
   13|      1|    }
   14|      1|
   15|      1|    if left < right {
   16|      1|        return -1;
   17|      0|    }
   18|      0|
   19|      0|    0
   20|      1|}
   21|       |
   22|       |#[no_mangle]
   23|      0|pub extern "C" fn compare_rev(left: i32, right: i32) -> i32 {
   24|      0|    return compare(right, left);
   25|      0|}
   26|       |
   27|       |#[no_mangle]
   28|       |#[no_coverage]
   29|       |pub extern "C" fn reset_cov() {
   30|       |    minicov::reset_coverage();
   31|       |}
   32|       |
   33|       |#[no_mangle]
   34|       |#[no_coverage]
   35|       |pub extern "C" fn output_cov() -> i32 {
   36|       |    const BUF_SIZE: usize = 4096;
   37|       |    let size = minicov::get_coverage_data_size();
   38|       |    if size > BUF_SIZE {
   39|       |        return -1;
   40|       |    }
   41|       |
   42|       |    let mut buf = [0u8; BUF_SIZE];
   43|       |    minicov::capture_coverage_to_buffer(&mut buf[..size]);
   44|       |
   45|       |    let ret = unsafe { my_output_profile(buf.as_ptr(), size as u32) };
   46|       |    if ret == -1 {
   47|       |        return -3;
   48|       |    }
   49|       |
   50|       |    0
   51|       |}
```


## how it works
1. provide a `profile runtime` for wasm.
2. collect profile data by running wasm inside a `wasmtime::Engine`.
3. use llvm toolchains to parse & output coverage report.

Since llvm won't put `__llvm_covmap` section into the compiled `.wasm` result,
we have to generate the LLVM-IR source file for taget `wasm32-unknown-unknown` and re-compile it into a `x86_64-unknown-linux-gnu` object file.

## related links
- [minicov](https://github.com/Amanieu/minicov)
- [WebAssembly Code Coverage](https://motley-coder.com/2019/03/31/webassembly-coverage/)
- [grcov](https://github.com/mozilla/grcov)
