// use std::env;
// use std::path::{Path, PathBuf};
// use std::process::Command;

mod imports {
    test_helpers::codegen_tinygo_import!(
        // TODO: implement the rest of the `*.wit` test suite
        "empty.wit"
        "smoke.wit"
    );
}

mod exports {
    test_helpers::codegen_tinygo_export!(
        // TODO: implement the rest of the `*.wit` test suite
        "empty.wit"
        "smoke.wit"
    );
}

fn verify(dir: &str, name: &str) {
    println!("dir: {}, name: {}", dir, name);
    // let (cmd, args) = ("go", &["get", "-d"] as &[&str]);
    // let go_get_status = Command::new(cmd).args(args).status().unwrap();
    // assert!(go_get_status.success());

    // let (cmd, args) = ("go", &["mod", "tidy"] as &[&str]);
    // let go_mod_status = Command::new(cmd).args(args).status().unwrap();
    // assert!(go_mod_status.success());

    // let (cmd, args) = ("tinygo", &["build", "-o"] as &[&str]);
    // let build_status = Command::new(cmd)
    //     .args(args)
    //     .arg(Path::new(dir).join(&format!("{}.wasm", name)))
    //     .arg("-target")
    //     .arg("wasi")
    //     .arg(Path::new(dir).join(&format!("{}.go", name)))
    //     .status()
    //     .unwrap();
    // assert!(build_status.success());
}
