use std::{error, fs, path::PathBuf, process::Command};

use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn error::Error>> {
    fetch_proto()
}

fn fetch_proto() -> Result<(), Box<dyn error::Error>> {
    let out_dir = ".proto";
    let buf_module = "buf.build/zako-ac/protofish";
    let prost_out_dir = "src/prost_generated";

    if !fs::exists(out_dir)? {
        Command::new("buf")
            .args(["export", buf_module, "--output", out_dir])
            .status()?;
    }

    let _ = fs::create_dir_all(prost_out_dir);

    let protos = list_protos(out_dir);
    println!("{:#?}", protos);

    prost_build::Config::new()
        .out_dir(prost_out_dir)
        .compile_protos(&protos, &[out_dir])?;

    Ok(())
}

fn list_protos(dir: &str) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| {
            let entry = e.ok()?;
            let path = entry.path();
            if path.extension()? == "proto" {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
        .collect()
}
