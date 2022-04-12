use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

mod imports {
    test_helpers::codegen_tinygo_import!(
        // TODO: implement async support
        "empty.wit"
    );
}

mod exports {
    test_helpers::codegen_tinygo_export!(
        // TODO: implement async support
        "empty.wit"
    );
}
