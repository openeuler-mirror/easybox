// spell-checker:ignore (vars) krate

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn main() {
    // println!("cargo:warning=Running build.rs");

    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", profile);
    }

    const ENV_FEATURE_PREFIX: &str = "CARGO_FEATURE_";
    const OVERRIDE_PREFIX: &str = "oe_";

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:warning=out_dir={}", out_dir);

    let mut crates = Vec::new();
    for (key, val) in env::vars() {
        if val == "1" && key.starts_with(ENV_FEATURE_PREFIX) {
            println!("cargo:warning=key={} {}", key, val);
            let krate = key[ENV_FEATURE_PREFIX.len()..].to_lowercase();
            match krate.as_ref() {
                "default" | "linux" => continue, // common/standard feature names
                "uudoc" => continue,             // is not a utility
                "test" => continue, // over-ridden with 'uu_test' to avoid collision with rust core crate 'test'
                _ => {}             // util feature name
            }
            crates.push(krate);
        }
    }
    crates.sort();

    let mut mf = File::create(Path::new(&out_dir).join("openeuler_map.rs")).unwrap();

    mf.write_all(
        "type UtilityMap<T> = phf::Map<&'static str, (fn(T) -> i32, fn() -> Command<'static>)>;\n\
         \n\
         fn util_map<T: uucore::Args>() -> UtilityMap<T> {\n"
            .as_bytes(),
    )
    .unwrap();

    let mut phf_map = phf_codegen::Map::<&str>::new();
    for krate in &crates {
        println!("cargo:warning=krate={}", krate);
        let map_value = format!("({krate}::oemain, {krate}::oe_app)", krate = krate);
        match krate.as_ref() {
            // 'test' is named uu_test to avoid collision with rust core crate 'test'.
            // It can also be invoked by name '[' for the '[ expr ] syntax'.
            "oe_test" => {
                phf_map.entry("test", &map_value);
                phf_map.entry("[", &map_value);
            }
            k if k.starts_with(OVERRIDE_PREFIX) => {
                phf_map.entry(&k[OVERRIDE_PREFIX.len()..], &map_value);
            }
            _ => {
                phf_map.entry(krate, &map_value);
            }
        }
    }
    write!(mf, "{}", phf_map.build()).unwrap();
    mf.write_all(b"\n}\n").unwrap();

    mf.flush().unwrap();
}
