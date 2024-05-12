pub mod dot;
pub mod fileutil;
pub mod graphviz;

#[derive(Debug)]
pub struct Edge {
    pub from: String,
    pub to: String,
}
