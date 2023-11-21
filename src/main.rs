use clap::Parser;
use std::collections::HashSet;
use std::io::{self, BufRead};

#[derive(Debug, Clone, clap::ValueEnum)]
enum Layout {
    Dot,
    Neato,
    Fdp,
    Sfdp,
    Circo,
    Twopi,
    Nop,
    Nop2,
    Osage,
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Dot
    }
}

impl ToString for Layout {
    fn to_string(&self) -> String {
        match self {
            Layout::Dot => "dot",
            Layout::Neato => "neato",
            Layout::Fdp => "fdp",
            Layout::Sfdp => "sfdp",
            Layout::Circo => "circo",
            Layout::Twopi => "twopi",
            Layout::Nop => "nop",
            Layout::Nop2 => "nop2",
            Layout::Osage => "osage",
        }.to_string()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    #[clap(default_value = ":")]
    delimiter: String,

    #[arg(short, long)]
    #[clap(default_value = "graph.svg")]
    output: String,

    #[arg(short, long)]
    #[clap(default_value = "G")]
    graph_name: String,

    #[arg(short, long, value_enum, default_value_t = Layout::default())]
    layout: Layout,
}

fn main() {
    env_logger::init();
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

    let mut graph_config = graphviz::Config::default();
    graph_config.name = args.graph_name;
    graph_config.graph.layout = args.layout.to_string();

    let (filename, mut dot_file) = fileutil::create_temp_file().expect("failed to create dot file");
    log::debug!("writing dot file to {}", filename.as_os_str().to_string_lossy());

    dot::write(&graph_config, &nodes, &edges, &mut dot_file)
        .expect("failed to write dot file");
    dot::compile(&args.output, &filename);
    println!("wrote {}", args.output);
}

mod graphviz {
    use std::io::{self, Write};

    pub struct Config {
        pub name: String,

        pub graph: GraphConfig,
        pub node: NodeConfig,
        pub edge: EdgeConfig,
    }

    impl Default for Config {
        fn default() -> Self {
            Config {
                name: "G".to_string(),
                graph: GraphConfig::default(),
                node: NodeConfig::default(),
                edge: EdgeConfig::default(),
            }
        }
    }

    impl Config {
        pub fn write(&self, file: &mut dyn Write) -> io::Result<()> {
            self.graph.write(file)?;
            self.node.write(file)?;
            self.edge.write(file)?;
            Ok(())
        }
    }

    pub struct GraphConfig {
        pub charset: String,
        pub layout: String,
    }

    impl Default for GraphConfig {
        fn default() -> Self {
            GraphConfig {
                charset: "UTF-8".to_string(),
                layout: "dot".to_string(),
            }
        }
    }

    impl GraphConfig {
        pub fn write(&self, file: &mut dyn Write) -> io::Result<()> {
            let indent = "  ";
            writeln!(file, "{}graph [", indent)?;
            writeln!(file, "{}{}charset=\"{}\";", indent, indent, self.charset)?;
            writeln!(file, "{}{}layout={};", indent, indent, self.layout)?;
            writeln!(file, "{}]", indent)?;
            Ok(())
        }
    }

    pub struct NodeConfig {
        pub shape: String,
    }

    impl Default for NodeConfig {
        fn default() -> Self {
            NodeConfig {
                shape: "box".to_string(),
            }
        }
    }

    impl NodeConfig {
        pub fn write(&self, file: &mut dyn Write) -> io::Result<()> {
            let indent = "  ";
            writeln!(file, "{}node [", indent)?;
            writeln!(file, "{}{}shape=\"{}\";", indent, indent, self.shape)?;
            writeln!(file, "{}]", indent)?;
            Ok(())
        }
    }

    pub struct EdgeConfig {
        pub arrowhead: String,
    }

    impl Default for EdgeConfig {
        fn default() -> Self {
            EdgeConfig {
                arrowhead: "normal".to_string(),
            }
        }
    }

    impl EdgeConfig {
        pub fn write(&self, file: &mut dyn Write) -> io::Result<()> {
            let indent = "  ";
            writeln!(file, "{}edge [", indent)?;
            writeln!(file, "{}{}arrowhead=\"{}\";", indent, indent, self.arrowhead)?;
            writeln!(file, "{}]", indent)?;
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct Edge {
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
    use crate::graphviz::{self};
    use crate::{fileutil, Edge};
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::{self, Write};

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
