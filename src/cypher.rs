use itertools::Itertools;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Value {
    Integer(i64),
    Double(String),
    String(String),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(integer) => write!(f, "{}", integer),
            Value::Double(double) => write!(f, "{}", double),
            Value::String(string) => write!(f, "\"{}\"", string),
            Value::Bool(bool) => write!(f, "{}", bool),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Properties {
    inner: HashMap<String, Option<Value>>,
}

impl Hash for Properties {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.inner.len());

        for (key, value) in self.inner.iter().sorted_by_key(|(&ref f, _)| f) {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl Display for Properties {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (k, v) in self.inner.iter() {
            if v.is_none() {
                continue;
            }
            let v = v.as_ref().unwrap();
            if !first {
                write!(f, ",")?;
            }
            first = false;
            write!(f, "{}:{}", k, v)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl Properties {
    fn new(inner: HashMap<String, Option<Value>>) -> Self {
        Self { inner }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Option<Value>)> {
        self.inner.iter().sorted_by_key(|(&ref t, _)| t)
    }

    pub fn get(&self, key: &str) -> Option<&Option<Value>> {
        self.inner.get(key)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Node {
    pub name: String,
    properties: Properties,
    primary_value: Value,
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(:{}{})", self.name, self.properties)
    }
}

impl Node {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Option<Value>)> {
        self.properties.iter()
    }

    fn new(name: String, properties: Properties) -> Self {
        let primary_value = properties.get("id").unwrap().clone().unwrap();
        Self {
            name,
            properties,
            primary_value,
        }
    }

    fn get_primary_value(&self) -> &Value {
        &self.primary_value
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Edge {
    pub name: String,
    pub properties: Option<Properties>,

    // pair of Node name and its primary value
    pub from: (String, Value),
    pub to: (String, Value),
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.properties {
            None => write!(f, "[:{}]", self.name)?,
            Some(ref props) => write!(f, "[:{}{}]", self.name, props)?,
        };
        Ok(())
    }
}

impl Edge {
    fn new(name: String, from: (String, Value), to: (String, Value)) -> Self {
        Self {
            name,
            properties: None,
            from,
            to,
        }
    }

    fn set_properties(&mut self, properties: HashMap<String, Option<Value>>) {
        self.properties = Some(Properties::new(properties));
    }
}

#[derive(Debug)]
pub struct Triple {
    pub left: Node,
    pub edge: Edge,
    pub right: Node,
}

impl Triple {
    fn new(left: Node, edge: Edge, right: Node) -> Self {
        Self { left, edge, right }
    }

    pub fn generate_create_statement(&self) -> String {
        let left = self.left.to_string();
        let edge = self.edge.to_string();
        let right = self.right.to_string();
        format!("CREATE {} -{}->{};", left, edge, right)
    }
}

fn parse_boolean_literal(pair: Pair<Rule>) -> bool {
    assert_eq!(pair.as_rule(), Rule::BooleanLiteral);

    let node = pair.into_inner().next().unwrap();
    match node.as_rule() {
        Rule::TRUE => true,
        Rule::FALSE => false,
        _ => unreachable!(),
    }
}

fn parse_number_literal(pair: Pair<Rule>) -> Value {
    assert_eq!(pair.as_rule(), Rule::NumberLiteral);

    let node = pair.into_inner().next().unwrap();
    match node.as_rule() {
        Rule::DoubleLiteral => Value::Double(node.as_str().to_string()),
        Rule::IntegerLiteral => Value::Integer(node.as_str().parse::<i64>().unwrap()),
        _ => unreachable!(),
    }
}

fn parse_string_literal(pair: Pair<Rule>) -> String {
    assert_eq!(pair.as_rule(), Rule::StringLiteral);

    let node = pair.into_inner().next().unwrap();
    match node.as_rule() {
        Rule::StringDoubleText => node.as_str().to_string(),
        Rule::StringSingleText => node.as_str().to_string(),
        _ => unreachable!(),
    }
}

fn parse_literal(pair: Pair<Rule>) -> Option<Value> {
    assert_eq!(pair.as_rule(), Rule::Literal);

    let node = pair.into_inner().next().unwrap();
    match node.as_rule() {
        Rule::BooleanLiteral => Some(Value::Bool(parse_boolean_literal(node))),
        Rule::NumberLiteral => Some(parse_number_literal(node)),
        Rule::StringLiteral => Some(Value::String(parse_string_literal(node))),
        Rule::MapLiteral => unimplemented!("not supported"),
        Rule::NULL => None,
        _ => unreachable!(),
    }
}

fn parse_map_literal(pair: Pair<Rule>) -> HashMap<String, Option<Value>> {
    assert_eq!(pair.as_rule(), Rule::MapLiteral);

    let mut map = HashMap::new();
    let mut current_key = String::new();
    for node in pair.into_inner() {
        match node.as_rule() {
            Rule::SP => continue,
            Rule::PropertyKeyName => {
                current_key = node.as_str().to_string();
            }
            Rule::Literal => {
                map.insert(current_key.clone(), parse_literal(node));
                current_key = String::new();
            }
            _ => unreachable!(),
        }
    }
    map
}

fn parse_properties(pair: Pair<Rule>) -> HashMap<String, Option<Value>> {
    assert_eq!(pair.as_rule(), Rule::Properties);
    parse_map_literal(pair.into_inner().next().unwrap())
}

fn parse_label_name(pair: Pair<Rule>) -> &str {
    assert_eq!(pair.as_rule(), Rule::LabelName);
    pair.as_str()
}

fn parse_node_label(pair: Pair<Rule>) -> &str {
    assert_eq!(pair.as_rule(), Rule::NodeLabel);

    let mut it = pair.into_inner();
    let node = it.next().unwrap();
    if matches!(node.as_rule(), Rule::LabelName) {
        return parse_label_name(node);
    }
    let node = it.next().unwrap();
    parse_label_name(node)
}

fn parse_node_pattern(pair: Pair<Rule>) -> Node {
    assert_eq!(pair.as_rule(), Rule::NodePattern);

    let mut name = String::new();
    let mut properties: HashMap<String, Option<Value>> = HashMap::new();
    for node in pair.into_inner() {
        match node.as_rule() {
            Rule::SP => continue,
            Rule::NodeLabel => name = parse_node_label(node).to_string(),
            Rule::Properties => properties = parse_properties(node),
            _ => unreachable!(),
        }
    }
    Node::new(name.trim().to_string(), Properties::new(properties))
}

fn parse_edge_label(pair: Pair<Rule>) -> &str {
    assert_eq!(pair.as_rule(), Rule::EdgeLabel);

    let mut it = pair.into_inner();
    let node = it.next().unwrap();
    if matches!(node.as_rule(), Rule::LabelName) {
        return parse_label_name(node);
    }
    let node = it.next().unwrap();
    parse_label_name(node)
}

fn parse_edge_pattern(pair: Pair<Rule>, left_node: &Node, right_node: &Node) -> Edge {
    assert_eq!(pair.as_rule(), Rule::EdgePattern);

    let mut name = String::new();
    let mut properties: HashMap<String, Option<Value>> = HashMap::new();
    for node in pair.into_inner() {
        match node.as_rule() {
            Rule::SP => continue,
            Rule::EdgeLabel => name = parse_edge_label(node).to_string(),
            Rule::Properties => properties = parse_properties(node),
            _ => unreachable!(),
        }
    }
    let mut edge = Edge::new(
        name.trim().to_string(),
        (
            left_node.name.to_string(),
            left_node.get_primary_value().clone(),
        ),
        (
            right_node.name.to_string(),
            right_node.get_primary_value().clone(),
        ),
    );
    if !properties.is_empty() {
        edge.set_properties(properties);
    }
    edge
}

// Node - Edge -> Node
fn parse_pattern_element(pair: Pair<Rule>) -> Triple {
    assert_eq!(pair.as_rule(), Rule::PatternElement);

    let mut elems = pair
        .into_inner()
        .filter(|p| !matches!(p.as_rule(), Rule::SP));

    // left node
    let left_pair = elems.next().unwrap();
    let left = parse_node_pattern(left_pair);

    // edge
    let mut is_right = true;
    let mut edge_pair = elems.next().unwrap();
    if matches!(edge_pair.as_rule(), Rule::LEFT_ARROW) {
        is_right = false;
        edge_pair = elems.next().unwrap();
    }

    // right node
    let mut right_pair = elems.next().unwrap();
    if is_right {
        right_pair = elems.next().unwrap();
    }
    let right = parse_node_pattern(right_pair);

    match is_right {
        true => {
            let edge = parse_edge_pattern(edge_pair, &left, &right);
            Triple::new(left, edge, right)
        }
        false => {
            let edge = parse_edge_pattern(edge_pair, &right, &left);
            Triple::new(right, edge, left)
        }
    }
}

fn parse_pattern(pair: Pair<Rule>) -> Triple {
    assert_eq!(pair.as_rule(), Rule::Pattern);

    parse_pattern_element(pair.into_inner().next().unwrap())
}

fn parse_pattern_list(pair: Pair<Rule>) -> Vec<Triple> {
    assert_eq!(pair.as_rule(), Rule::PatternList);

    pair.into_inner()
        .filter(|p| matches!(p.as_rule(), Rule::Pattern))
        .map(|p| parse_pattern(p))
        .collect::<Vec<_>>()
}

#[derive(Parser)]
#[grammar = "cypher.pest"]
struct CypherParser;

pub fn parse(input: &str) -> anyhow::Result<Vec<Triple>> {
    let pairs = CypherParser::parse(Rule::CypherLike, &input)?;

    let mut triples = Vec::new();
    for pair in pairs {
        for line in pair.into_inner() {
            match line.as_rule() {
                Rule::PatternList => {
                    triples.append(&mut parse_pattern_list(line));
                }
                Rule::SP => continue,
                Rule::EOI => break,
                _ => unreachable!(),
            }
        }
    }
    Ok(triples)
}

#[derive(Debug)]
pub enum FieldType {
    Integer,
    Double,
    Boolean,
    String,
}

impl Display for FieldType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            FieldType::Integer => write!(f, "INT64"),
            FieldType::Double => write!(f, "DOUBLE"),
            FieldType::Boolean => write!(f, "BOOLEAN"),
            FieldType::String => write!(f, "STRING"),
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub r#type: FieldType,
    pub nullable: bool,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum TableType {
    Node,
    Edge(String, String), // (from, to)
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub r#type: TableType,
    pub fields: HashMap<String, Field>,
    pub primary_key: String,
}

impl Table {
    pub fn iter_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields
            .iter()
            .sorted_by_key(|(&ref t, _)| t)
            .map(|(_, &ref f)| f)
    }

    fn add_field(&mut self, name: &str, r#type: FieldType, nullable: bool) {
        self.fields
            .entry(name.to_string())
            .or_insert_with(|| Field {
                name: name.to_string(),
                r#type,
                nullable,
            });
    }

    fn set_field_nullable(&mut self, name: &str, nullable: bool) {
        self.fields.get_mut(name).unwrap().nullable |= nullable;
    }

    fn merge_properties(&mut self, properties: &Properties) {
        for (k, v) in properties.iter() {
            match v {
                Some(Value::Integer(_)) => self.add_field(&k, FieldType::Integer, false),
                Some(Value::Double(_)) => self.add_field(&k, FieldType::Double, false),
                Some(Value::String(_)) => self.add_field(&k, FieldType::String, false),
                Some(Value::Bool(_)) => self.add_field(&k, FieldType::Boolean, false),
                None => {
                    self.add_field(&k, FieldType::String, true);
                    self.set_field_nullable(&k, true);
                }
            }
        }
    }

    fn generate_fields(&self) -> String {
        let fields = self
            .fields
            .iter()
            .sorted_by_key(|(&ref t, _)| t)
            .map(|(k, v)| format!("{} {}", k, v.r#type))
            .collect::<Vec<_>>()
            .join(", ");

        match self.r#type {
            TableType::Node => {
                format!("({}, PRIMARY KEY ({}))", fields, self.primary_key)
            }
            TableType::Edge(ref from, ref to) => format!("(FROM {} TO {}, {})", from, to, fields),
        }
    }

    pub fn generate_create_statement(&self) -> String {
        let (ty, fields) = match self.r#type {
            TableType::Node => ("NODE", self.generate_fields()),
            TableType::Edge(..) => ("REL", self.generate_fields()),
        };
        format!("CREATE {} TABLE {} {};", ty, self.name, fields)
    }
}

pub struct Schema {
    tables: HashMap<String, Table>,
}

impl Schema {
    pub fn new() -> Self {
        Schema {
            tables: HashMap::new(),
        }
    }

    pub fn iter_table(&self) -> impl Iterator<Item = &Table> {
        self.tables
            .iter()
            .sorted_by_key(|(_, &ref v)| v.r#type.clone())
            .map(|(_, &ref v)| v)
    }

    pub fn get(&self, table_name: &str) -> Option<&Table> {
        self.tables.get(table_name)
    }
}

fn extract_table_from_node(schema: &mut Schema, node: &Node) {
    let table = schema.tables.entry(node.name.clone()).or_insert(Table {
        name: node.name.clone(),
        r#type: TableType::Node,
        fields: HashMap::new(),
        primary_key: "id".to_string(),
    });
    assert_eq!(table.r#type, TableType::Node);
    table.merge_properties(&node.properties);
}

fn extract_schema_from_edge(schema: &mut Schema, edge: &Edge) {
    let table = schema.tables.entry(edge.name.clone()).or_insert(Table {
        name: edge.name.clone(),
        r#type: TableType::Edge(edge.from.0.clone(), edge.to.0.clone()),
        fields: HashMap::new(),
        primary_key: "id".to_string(),
    });
    assert!(matches!(table.r#type, TableType::Edge(..)));

    if let Some(properties) = &edge.properties {
        table.merge_properties(properties);
    }
}

pub fn extract_schema(triples: &[Triple]) -> Schema {
    let mut schema = Schema::new();
    for triple in triples {
        extract_table_from_node(&mut schema, &triple.left);
        extract_schema_from_edge(&mut schema, &triple.edge);
        extract_table_from_node(&mut schema, &triple.right);
    }
    schema
}
