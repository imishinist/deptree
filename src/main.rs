use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::{error, fs, mem};

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use deptree::{cypher, dot, fileutil, graphviz, Edge, Graph};
use itertools::Itertools;
use kuzu::{Connection, Database, SystemConfig};

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

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Layout::Dot => "dot",
            Layout::Neato => "neato",
            Layout::Fdp => "fdp",
            Layout::Sfdp => "sfdp",
            Layout::Circo => "circo",
            Layout::Twopi => "twopi",
            Layout::Nop => "nop",
            Layout::Nop2 => "nop2",
            Layout::Osage => "osage",
        };
        write!(f, "{}", str)
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

impl Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Shape::Box => "box",
            Shape::Ellipse => "ellipse",
            Shape::Oval => "oval",
            Shape::Circle => "circle",
            Shape::Point => "point",
            Shape::Triangle => "triangle",
            Shape::Diamond => "diamond",
            Shape::Record => "record",
        };
        write!(f, "{}", str)
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
    Kuzu(KuzuCommand),
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
            let (from, to, label) = parse_line(input, &self.edge_delimiter, &self.label_delimiter)
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

#[derive(Args, Debug)]
struct KuzuCommand {
    output: String,
}

impl KuzuCommand {
    fn run(&self) -> anyhow::Result<()> {
        let output = Path::new(&self.output);
        if output.exists() {
            return Err(anyhow::anyhow!("{} already exists", self.output));
        }

        fs::create_dir_all(output)?;
        let db = Database::new(output, SystemConfig::default())?;
        let conn = Connection::new(&db)?;

        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        let triples = cypher::parse(&input)?;
        let schema = cypher::extract_schema(&triples);
        for table in schema.iter_table() {
            let stmt = table.generate_create_statement();
            log::info!("{}", stmt);
            conn.query(&stmt)?;
        }

        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        for triple in triples {
            nodes
                .entry(triple.left.name.clone())
                .or_insert_with(HashSet::new)
                .insert(triple.left);
            nodes
                .entry(triple.right.name.clone())
                .or_insert_with(HashSet::new)
                .insert(triple.right);
            edges
                .entry(triple.edge.name.clone())
                .or_insert_with(HashSet::new)
                .insert(triple.edge);
        }

        let tmp = tempfile::tempdir()?;

        // write nodes
        for (table_name, nodes) in &nodes {
            let file_name = format!("{}.csv", table_name);
            let path = tmp.path().join(&file_name);
            let mut file = fs::File::create(&path)?;

            log::info!("setup {}.csv", file_name);
            for node in nodes {
                // TODO: null value
                let values = node
                    .iter()
                    .map(|(_, v)| v.as_ref().map(|v| v.to_string()).unwrap_or("".to_string()))
                    .join(",");
                log::debug!("{}", values);
                writeln!(file, "{}", values)?;
            }

            let query = format!("COPY {} FROM '{}'", table_name, path.display());
            log::info!("{}", query);
            conn.query(&query)?;
        }

        // write edges
        for (table_name, edges) in &edges {
            let file_name = format!("{}.csv", table_name);
            let path = tmp.path().join(&file_name);

            let mut file = fs::File::create(&path)?;
            log::info!("setup {}.csv", file_name);
            for edge in edges {
                write!(file, "{},{}", edge.from.1, edge.to.1)?;
                match edge.properties {
                    Some(ref properties) => {
                        // TODO: null value
                        let values = properties
                            .iter()
                            .map(|(_, v)| {
                                v.as_ref().map(|v| v.to_string()).unwrap_or("".to_string())
                            })
                            .join(",");
                        writeln!(file, ",{}", values)?;
                    }
                    None => writeln!(file, "")?,
                }
            }
            let query = format!("COPY {} FROM '{}'", table_name, path.display());
            log::info!("{}", query);
            conn.query(&query)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();

    let deptree = DepTreeCommands::parse();
    match deptree.commands {
        Commands::Graph(graph) => graph.run()?,
        Commands::Kuzu(kuzu) => kuzu.run()?,
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
