use std::io::Result;
use std::path::{Path, PathBuf};

pub fn get_libs_search_paths(
    target_dir: &Path,
    target_triple: &str,
    target_profile: &Path,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    let deps_dir = target_dir
        .join(target_triple)
        .join(target_profile)
        .join("build");

    for dep_dir in deps_dir.read_dir()? {
        let output_file = dep_dir?.path().join("output");
        if output_file.is_file() {
            use std::{
                fs::File,
                io::{BufRead, BufReader},
            };
            for line in BufReader::new(File::open(output_file)?).lines() {
                let line = line?;
                if let Some(link_search) = line.strip_prefix("cargo:rustc-link-search=") {
                    let mut pie = link_search.split('=');
                    let (kind, path) = match (pie.next(), pie.next()) {
                        (Some(kind), Some(path)) => (kind, path),
                        (Some(path), None) => ("all", path),
                        _ => unreachable!(),
                    };
                    match kind {
                        // FIXME: which kinds of search path we interested in
                        "dependency" | "native" | "all" => paths.push(path.into()),
                        _ => (),
                    };
                }
            }
        }
    }

    Ok(paths)
}
