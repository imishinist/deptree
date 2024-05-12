use std::io::{self, BufRead};
use std::{error, mem};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use deptree::{dot, fileutil, graphviz, Edge, Graph};

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

#[derive(Debug, Clone, clap::ValueEnum, Default)]
enum Shape {
    #[default]
    Box,
    Ellipse,
    Oval,
    Circle,
    Point,
    Triangle,
    Diamond,
    Record,
}

impl ToString for Shape {
    fn to_string(&self) -> String {
        match self {
            Shape::Box => "box",
            Shape::Ellipse => "ellipse",
            Shape::Oval => "oval",
            Shape::Circle => "circle",
            Shape::Point => "point",
            Shape::Triangle => "triangle",
            Shape::Diamond => "diamond",
            Shape::Record => "record",
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
    #[arg(long)]
    #[clap(default_value = "->")]
    edge_delimiter: String,

    #[arg(long)]
    #[clap(default_value = ":")]
    label_delimiter: String,

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

    #[arg(short, long, value_enum, default_value_t = Shape::default())]
    node_shape: Shape,
}

impl GraphCommand {
    fn run(&self) -> anyhow::Result<()> {
        let inputs = read_input().context("failed to read input")?;

        let mut graph = Graph::new();
        for (idx, input) in inputs.iter().enumerate() {
            let (from, to, label) =
                parse_line(input, &self.edge_delimiter, &self.label_delimiter)
                    .with_context(|| format!("error parsing line {}: \"{}\"", idx + 1, input))?;
            let mut from_id = graph.insert_node(from);
            let mut to_id = graph.insert_node(to);
            if self.reverse {
                mem::swap(&mut from_id, &mut to_id);
            }

            let edge = Edge {
                from: from_id,
                to: to_id,
                label: label.map(|s| s.to_string()),
            };
            graph.add_edge(edge);
        }

        let mut graph_config = graphviz::Config {
            name: self.graph_name.clone(),
            ..Default::default()
        };
        graph_config.graph.layout = self.layout.to_string();
        graph_config.node.shape = self.node_shape.to_string();

        let (filename, mut dot_file) =
            fileutil::create_temp_file().context("failed to create temp file")?;
        log::debug!(
            "writing dot file to {}",
            filename.as_os_str().to_string_lossy()
        );

        dot::write(&graph_config, &graph, &mut dot_file)
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

fn parse_line<'a>(
    line: &'a str,
    edge_delim: &str,
    label_delim: &str,
) -> Option<(&'a str, &'a str, Option<&'a str>)> {
    // parse line
    // a->b:foo

    let mut split = line.split(label_delim);
    let edge = split.next()?;
    let label = split.next();

    let mut split = edge.split(edge_delim);
    Some((split.next()?, split.next()?, label))
}

fn read_input() -> io::Result<Vec<String>> {
    let mut ret = Vec::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        ret.push(line?);
    }
    Ok(ret)
}
