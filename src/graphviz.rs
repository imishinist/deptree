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
        writeln!(
            file,
            "{}{}arrowhead=\"{}\";",
            indent, indent, self.arrowhead
        )?;
        writeln!(file, "{}]", indent)?;
        Ok(())
    }
}
