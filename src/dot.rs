use std::fs::File;
use std::io::{self, Write};

use crate::fileutil;
use crate::graphviz;

const DEFAULT_OUTPUT_FORMAT: &str = "svg";

pub fn write(
    graph_config: &graphviz::Config,
    graph: &crate::Graph,
    file: &mut File,
) -> io::Result<()> {
    writeln!(file, "digraph {} {{", graph_config.name)?;
    graph_config.write(file)?;

    for node in graph.node_arena.nodes.iter().enumerate() {
        writeln!(file, "    N_{} [label=\"{}\"];", node.0, node.1)?;
    }
    for edge in graph.edges.iter() {
        writeln!(file, "    N_{} -> N_{};", edge.from, edge.to)?;
    }
    writeln!(file, "}}")?;
    Ok(())
}

pub fn compile(output_file: &str, filename: &std::path::Path) -> anyhow::Result<()> {
    let extension = fileutil::get_extension(output_file).unwrap_or(DEFAULT_OUTPUT_FORMAT);
    // dot -T${extension} -o ${args.output} ${filename}
    let output = std::process::Command::new("dot")
        .arg(format!("-T{}", extension))
        .arg("-o")
        .arg(output_file)
        .arg(filename.as_os_str())
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "dot failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
