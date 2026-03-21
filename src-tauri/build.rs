use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // Generate embedded lensfun XML data
    let data_dir = Path::new("data/lensfun");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("lensfun_data.rs");
    let mut f = fs::File::create(&dest).unwrap();

    writeln!(f, "/// Auto-generated: embedded lensfun XML database").unwrap();
    writeln!(f, "pub static LENSFUN_XML: &[&str] = &[").unwrap();

    if data_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(data_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "xml"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in &entries {
            let path = entry.path();
            let abs = fs::canonicalize(&path).unwrap();
            let abs_str = abs.to_str().unwrap().replace('\\', "/");
            writeln!(f, "    include_str!(\"{abs_str}\"),").unwrap();
        }

        println!("cargo:warning=Embedded {} lensfun XML files", entries.len());
    } else {
        println!("cargo:warning=No lensfun data directory found at data/lensfun/");
    }

    writeln!(f, "];").unwrap();
    println!("cargo:rerun-if-changed=data/lensfun");

    tauri_build::build()
}
