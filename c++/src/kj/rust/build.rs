use eyre::eyre;
use eyre::Result;
use kj_build::BuildExt;
use std::{env, fs, path::Path};

const CAPNP_HEAVY: bool = cfg!(feature = "heavy");
const USE_LIBDL: bool = cfg!(feature = "libdl");
const USE_SAVE_ACQUIRED_LOCK_INFO: bool = cfg!(feature = "save_acquired_lock_info");
const USE_TRACK_LOCK_BLOCKING: bool = cfg!(feature = "track_lock_blocking");

static KJ_SOURCES_LITE: &[&str] = &[
    "arena.c++",
    "array.c++",
    //"cidr.c++",
    "common.c++",
    "debug.c++",
    "encoding.c++",
    "exception.c++",
    //"glob-filter.c++",
    "hash.c++",
    "io.c++",
    "list.c++",
    "main.c++",
    "memory.c++",
    "mutex.c++",
    "source-location.c++",
    "string.c++",
    "table.c++",
    //"test-helpers.c++",
    "thread.c++",
    "units.c++",
];
static KJ_SOURCES_HEAVY: &[&str] = &[
    "filesystem.c++",
    "filesystem-disk-unix.c++",
    "filesystem-disk-win32.c++",
    "parse/char.c++",
    "refcount.c++",
    "string-tree.c++",
    "time.c++",
];
static KJ_HEADERS: &[&str] = &["arena.h", "common.h", "list.h"];
static KJ_PRIVATE_HEADERS: &[&str] = &[
    "array.h",
    "cidr.h",
    "debug.h",
    "encoding.h",
    "exception.h",
    "filesystem.h",
    "function.h",
    "glob-filter.h",
    "hash.h",
    "io.h",
    "main.h",
    "map.h",
    "memory.h",
    "miniposix.h",
    "mutex.h",
    "one-of.h",
    "refcount.h",
    "source-location.h",
    "string.h",
    "string-tree.h",
    "table.h",
    "test.h",
    "thread.h",
    "time.h",
    "timer.h",
    "tuple.h",
    "units.h",
    "vector.h",
    "win32-api-version.h",
    "windows-sanity.h",
];
static KJ_PARSE_HEADERS: &[&str] = &["parse/common.h", "parse/char.h"];
static KJ_STD_HEADERS: &[&str] = &["std/iostream.h"];

fn bool_int_str(p: bool) -> &'static str {
    if p {
        "1"
    } else {
        "0"
    }
}

fn kj_configure<'a>(build: &'a mut cc::Build, kj_cfg: &mut kj_build::Cfg) -> &'a mut cc::Build {
    kj_cfg
        .define_propagated("KJ_CONTENTION_WARNING_THRESHOLD", "100")
        .define_propagated(
            "KJ_SAVE_ACQUIRED_LOCK_INFO",
            bool_int_str(USE_SAVE_ACQUIRED_LOCK_INFO),
        )
        .define_propagated(
            "KJ_TRACK_LOCK_BLOCKING",
            bool_int_str(USE_TRACK_LOCK_BLOCKING),
        );
    if USE_LIBDL {
        build.define("KJ_HAS_LIBDL", None);
    }
    build
}

fn capnp_configure<'a>(build: &'a mut cc::Build, kj_cfg: &mut kj_build::Cfg) -> &'a mut cc::Build {
    kj_configure(build, kj_cfg);
    if !CAPNP_HEAVY {
        kj_cfg.define_propagated("CAPNP_LITE", None);
    }
    build
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| eyre!("OUT_DIR not set"))?;
    let headers = Path::new(&out_dir).join("headers");
    let sources = Path::new(&out_dir).join("sources");

    let source_dir = Path::new("..");

    let kj_header_dir = headers.join("kj");
    let kj_source_dir = sources.join("kj");
    fs::create_dir_all(&kj_source_dir)?;
    fs::create_dir_all(kj_source_dir.join("parse"))?;
    fs::create_dir_all(kj_source_dir.join("std"))?;
    fs::create_dir_all(&kj_header_dir)?;
    fs::create_dir_all(kj_header_dir.join("parse"))?;
    fs::create_dir_all(kj_header_dir.join("std"))?;
    for kj_header in KJ_HEADERS
        .into_iter()
        .chain(KJ_PARSE_HEADERS)
        .chain(KJ_STD_HEADERS)
    {
        let kj_header_file = source_dir.join(kj_header);
        println!(
            "cargo:rerun-if-changed={}",
            kj_header_file
                .to_str()
                .ok_or_else(|| eyre!("non–UTF-8 path: {:?}", kj_header_file))?
        );
        fs::copy(&*kj_header_file, &*kj_header_dir.join(kj_header))?;
    }

    for kj_private_header in KJ_PRIVATE_HEADERS {
        let kj_private_header_file = source_dir.join(kj_private_header);
        println!(
            "cargo:rerun-if-changed={}",
            kj_private_header_file
                .to_str()
                .ok_or_else(|| eyre!("non–UTF-8 path: {:?}", kj_private_header_file))?
        );
        fs::copy(
            &*kj_private_header_file,
            &*kj_source_dir.join(kj_private_header),
        )?;
    }

    cxx_build::CFG.exported_header_dirs.push(&headers);
    cxx_build::CFG.include_prefix = "kj";
    let mut build = cxx_build::bridge("lib.rs");
    let mut kj_cfg = kj_build::Cfg::default();
    kj_cfg.import_propagated_definitions()?;

    fs::create_dir_all(kj_source_dir.join("parse"))?;
    fs::create_dir_all(kj_source_dir.join("std"))?;
    for kj_source in KJ_SOURCES_LITE {
        let kj_source_file = source_dir.join(kj_source);
        println!(
            "cargo:rerun-if-changed={}",
            kj_source_file
                .to_str()
                .ok_or_else(|| eyre!("non–UTF-8 path: {:?}", kj_source_file))?
        );
        let hermetic_kj_source = kj_source_dir.join(kj_source);
        fs::copy(&*kj_source_file, &*hermetic_kj_source)?;
        build.file(hermetic_kj_source);
    }
    if CAPNP_HEAVY {
        for kj_source in KJ_SOURCES_HEAVY {
            let kj_source_file = source_dir.join(kj_source);
            println!(
                "cargo:rerun-if-changed={}",
                kj_source_file
                    .to_str()
                    .ok_or_else(|| eyre!("non–UTF-8 path: {:?}", kj_source_file))?
            );
            let hermetic_kj_source = kj_source_dir.join(kj_source);
            fs::copy(&*kj_source_file, &*hermetic_kj_source)?;
            build.file(hermetic_kj_source);
        }
    }
    capnp_configure(&mut build, &mut kj_cfg);
    println!("cargo:rustc-link-lib=pthread");
    if USE_LIBDL {
        println!("cargo:rustc-link-lib=dl");
    }

    // Unfuck MSVC
    build.flag_if_supported("/Zc:__cplusplus");
    build.flag_if_supported("/EHsc");
    build.flag_if_supported("/TP");
    build.include(kj_header_dir);
    build.std("c++20").propagate_definitions(&mut kj_cfg)?;
    println!("BUILD: {:?}", build);
    build.compile("kj");

    Ok(())
}
