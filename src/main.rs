use clap::Parser;
use std::collections::HashSet;
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    #[clap(default_value = ":")]
    delimiter: String,

    #[arg(short, long)]
    #[clap(default_value = "graph.svg")]
    output: String,
}

fn main() {
    let args = Args::parse();

    let inputs = read_input().expect("failed to read input");

    let mut nodes = HashSet::new();
    let mut edges = Vec::new();
    for (idx, input) in inputs.iter().enumerate() {
        match parse_line(&input, &args.delimiter) {
            Some(edge) => {
                nodes.insert(edge.from.clone());
                nodes.insert(edge.to.clone());
                edges.push(edge);
            }
            None => {
                eprintln!("error parsing line {}: \"{}\"", idx + 1, input);
                std::process::exit(1);
            }
        }
    }

    let (filename, mut dot_file) = fileutil::create_temp_file().expect("failed to create dot file");
    dot::write(&nodes, &edges, &mut dot_file).expect("failed to write dot file");
    dot::compile(&args.output, &filename);
}

#[derive(Debug)]
struct Edge {
    from: String,
    to: String,
}

fn parse_line(line: &str, delim: &str) -> Option<Edge> {
    let mut split = line.split(delim);
    let from = split.next()?.to_string();
    let to = split.next()?.to_string();
    Some(Edge { from, to })
}

fn read_input() -> io::Result<Vec<String>> {
    let mut ret = Vec::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        ret.push(line?);
    }
    Ok(ret)
}

mod dot {
    use crate::{fileutil, Edge};
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::{self, Write};

    pub fn write(nodes: &HashSet<String>, edges: &Vec<Edge>, file: &mut File) -> io::Result<()> {
        writeln!(file, "digraph G {{")?;
        for node in nodes {
            writeln!(file, "    {};", node)?;
        }
        for edge in edges {
            writeln!(file, "    {} -> {};", edge.from, edge.to)?;
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
}

mod fileutil {
    use std::fs::File;
    use std::path::PathBuf;
    use std::{io, mem};

    pub fn get_extension(filename: &str) -> &str {
        let mut split = filename.split('.');
        split.next_back().unwrap_or("")
    }

    pub fn create_temp_file() -> io::Result<(PathBuf, File)> {
        let dir = tempfile::tempdir()?;
        let filename = dir.path().join("graph.dot");
        let file = File::create(&filename)?;

        // file is leaked, but that's ok
        mem::forget(dir);
        Ok((filename, file))
    }
}
