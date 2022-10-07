use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_descriptor_set_path = Path::new("vasystem-api.protoset");

    let proto_paths: Vec<PathBuf> = vec![];
    let includes: Vec<PathBuf> = vec![];

    println!(
        "cargo:rerun-if-changed={}",
        file_descriptor_set_path.display()
    );

    let mut config = prost_build::Config::new();
    config
        .file_descriptor_set_path(file_descriptor_set_path)
        .skip_protoc_run()
        .service_generator(
            tonic_build::configure()
                .build_client(true)
                .build_server(false)
                .service_generator(),
        )
        .compile_protos(&proto_paths, &includes)?;

    Ok(())
}
