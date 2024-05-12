use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};

use crate::graphviz;
use crate::{fileutil, Edge};

pub fn write(
    graph_config: &graphviz::Config,
    nodes: &HashSet<String>,
    edges: &Vec<Edge>,
    file: &mut File,
) -> io::Result<()> {
    writeln!(file, "digraph {} {{", graph_config.name)?;
    graph_config.write(file)?;

    for node in nodes {
        writeln!(file, "    \"{}\";", node)?;
    }
    for edge in edges {
        writeln!(file, "    \"{}\" -> \"{}\";", edge.from, edge.to)?;
    }
    writeln!(file, "}}")?;
    Ok(())
}

pub fn compile(output_file: &str, filename: &std::path::Path) {
    // dot -Tsvg -o ${args.output} ${filename}
    let output = std::process::Command::new("dot")
        .arg(format!("-T{}", fileutil::get_extension(output_file)))
        .arg("-o")
        .arg(output_file)
        .arg(filename.as_os_str())
        .output()
        .expect("failed to execute dot");
    if !output.status.success() {
        eprintln!("dot failed: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}
