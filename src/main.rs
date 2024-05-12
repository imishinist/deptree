use anyhow::Context;
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::{error, mem};

use clap::{Args, Parser, Subcommand};
use deptree::{dot, fileutil, graphviz, Edge};

#[derive(Debug, Clone, clap::ValueEnum, Default)]
enum Layout {
    #[default]
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
        }
        .to_string()
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct DepTreeCommands {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Graph(GraphCommand),
}

#[derive(Args, Debug)]
struct GraphCommand {
    #[arg(short, long)]
    #[clap(default_value = ":")]
    delimiter: String,

    #[arg(short, long)]
    #[clap(default_value = "graph.svg")]
    output: String,

    #[arg(short, long)]
    #[clap(default_value = "G")]
    graph_name: String,

    #[arg(short, long)]
    #[clap(default_value_t = false)]
    reverse: bool,

    #[arg(short, long, value_enum, default_value_t = Layout::default())]
    layout: Layout,
}

impl GraphCommand {
    fn run(&self) -> anyhow::Result<()> {
        let inputs = read_input().context("failed to read input")?;

        let mut nodes = HashSet::new();
        let mut edges = Vec::new();
        for (idx, input) in inputs.iter().enumerate() {
            let mut edge = parse_line(input, &self.delimiter)
                .with_context(|| format!("error parsing line {}: \"{}\"", idx + 1, input))?;
            if self.reverse {
                mem::swap(&mut edge.from, &mut edge.to);
            }
            nodes.insert(edge.from.clone());
            nodes.insert(edge.to.clone());
            edges.push(edge);
        }

        let mut graph_config = graphviz::Config {
            name: self.graph_name.clone(),
            ..Default::default()
        };
        graph_config.graph.layout = self.layout.to_string();

        let (filename, mut dot_file) =
            fileutil::create_temp_file().context("failed to create temp file")?;
        log::debug!(
            "writing dot file to {}",
            filename.as_os_str().to_string_lossy()
        );

        dot::write(&graph_config, &nodes, &edges, &mut dot_file)
            .context("failed to write temporary dot file")?;
        dot::compile(&self.output, &filename).context("failed to compile temporary dot file")?;
        println!("wrote {}", self.output);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();

    let deptree = DepTreeCommands::parse();
    match deptree.commands {
        Commands::Graph(graph) => graph.run()?,
    }
    Ok(())
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
