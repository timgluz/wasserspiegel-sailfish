use std::fs;
use std::path::PathBuf;

fn main() {
    let mut bridge = cxx_build::bridge("src/lib.rs");
    bridge
        .flag_if_supported("-std=c++17")
        .compile("wasserspiegel-core");

    // Expose the generated bridge header and rust/cxx.h at a stable
    // location so the qmake side can simply use -I rust/include.
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let inc_dir = manifest_dir.join("include");
    let rust_inc = inc_dir.join("rust");
    fs::create_dir_all(&rust_inc).expect("create rust/include/rust");

    let header_src = out_dir.join("cxxbridge/include/wasserspiegel-core/src/lib.rs.h");
    let bridge_dst = inc_dir.join("wasserspiegel_bridge.h");
    fs::copy(&header_src, &bridge_dst).expect("copy bridge header");

    let cxx_h_src = out_dir.join("cxxbridge/include/rust/cxx.h");
    fs::copy(&cxx_h_src, rust_inc.join("cxx.h")).expect("copy rust/cxx.h");

    // Umbrella header: rust/cxx.h first, then the generated bridge
    // (the generated header inlines only a trimmed cxx.h subset).
    fs::write(
        inc_dir.join("wasserspiegel_core.h"),
        "#pragma once\n#include \"rust/cxx.h\"\n#include \"wasserspiegel_bridge.h\"\n",
    )
    .expect("write umbrella header");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/client.rs");
}
