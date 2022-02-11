use wasmtime;

fn main() {
    let engine = wasmtime::Engine::default();
    let module =
        wasmtime::Module::from_file(&engine, "./dist/actor.wasm").expect("load actor.wasm");

    let mut linker = wasmtime::Linker::<()>::new(&engine);

    // these funcs are required by the generated profiling code, but not used in minicov
    linker
        .func_wrap(
            "env",
            "__llvm_profile_register_names_function",
            |_name_ptr: u32, _name_size: u32| {},
        )
        .expect("inject __llvm_profile_register_names_function");

    linker
        .func_wrap("env", "__llvm_profile_register_function", |_data: u32| {})
        .expect("inject __llvm_profile_register_function");

    // help to output profile
    linker
        .func_wrap(
            "my_profile",
            "my_output_profile",
            |mut caller: wasmtime::Caller<'_, _>, bytes_ptr: u32, bytes_len: u32| -> i32 {
                let ret = caller
                    .get_export("memory")
                    .and_then(|m| m.into_memory())
                    .and_then(|mem| {
                        let mut buf = vec![0u8; bytes_len as usize];
                        match mem.read(&caller, bytes_ptr as usize, &mut buf[..]) {
                            Ok(_) => {
                                println!("READ {} data", bytes_len);
                                std::fs::write("./dist/actor.profraw", &buf[..])
                                    .map_err(|e| {
                                        println!("failed to write prof raw: {:?}", e);
                                    })
                                    .ok()
                            }

                            Err(e) => {
                                println!("failed to read mem: {:?}", e);
                                None
                            }
                        }
                    });

                if ret.is_none() {
                    return -1;
                }

                0
            },
        )
        .expect("inject my_profile::my_output_profile");

    let mut store = wasmtime::Store::new(&engine, ());

    let instance = linker
        .instantiate(&mut store, &module)
        .expect("instantiate runtime");

    let my_cmp: wasmtime::TypedFunc<(i32, i32), i32> = instance
        .get_typed_func(&mut store, "compare")
        .expect("get compare fn");

    let my_cmp_rev: wasmtime::TypedFunc<(i32, i32), i32> = instance
        .get_typed_func(&mut store, "compare_rev")
        .expect("get compare_rev fn");

    let reset_fn: wasmtime::TypedFunc<(), ()> = instance
        .get_typed_func(&mut store, "reset_cov")
        .expect("get reset_cov");

    let output_fn: wasmtime::TypedFunc<(), i32> = instance
        .get_typed_func(&mut store, "output_cov")
        .expect("get output_cov");

    reset_fn.call(&mut store, ()).expect("call reset_cov");

    let num = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("get now sec")
        .as_secs() as i32;

    let args: Vec<String> = std::env::args().collect();
    let (rev, first_args) = if let Some(true) = args.get(1).map(|s| s == "rev") {
        (true, 2)
    } else {
        (false, 1)
    };

    let left = args
        .get(first_args)
        .and_then(|s| s.parse().ok())
        .unwrap_or(num);

    let right = args
        .get(first_args + 1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(num);

    let ret = if rev {
        my_cmp_rev.call(&mut store, (left, right))
    } else {
        my_cmp.call(&mut store, (left, right))
    }
    .expect("call func failed");

    println!(
        "compare: rev={} left = {}, right = {}, ret = {}",
        rev, left, right, ret
    );

    let ret = output_fn.call(&mut store, ()).expect("call output_fn");
    println!("output with ret: {}", ret);
}
