use std::path::{Path};

use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_path = Path::new(".");
    let pb_path = Path::new("pb");

    // Collect all proto files
    let proto_entries = WalkDir::new(pb_path)
        .into_iter()
        .filter(|e| {
            match e {
                Err(e) => panic!("{}", e),
                Ok(result) => result.file_type().is_file() && result.path().extension().unwrap_or("".as_ref()) == "proto"
            }
        })
        .collect::<Result<Vec<_>, walkdir::Error>>()?;
    let proto_paths = proto_entries.into_iter().map(|e| e.into_path()).collect::<Vec<_>>();

    for path in &proto_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(&proto_paths, &[root_path])?;

    Ok(())
}
