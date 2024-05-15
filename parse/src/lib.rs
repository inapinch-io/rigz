#[macro_use]
extern crate pest_derive;

use anyhow::{anyhow, Result};
use pest::iterators::Pairs;
use pest::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Parser)]
#[grammar = "src/grammar.pest"]
struct Tokenizer;

#[derive(Default)]
pub struct ParseConfig {
    pub use_64_bit_numbers: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ASTFunctionCall {
    pub identifier: Identifier,
    pub args: Vec<Element>,
    pub definition: Option<Definition>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Definition {
    Object(Object),
    List(List),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for (k, v) in &self.0 {
            let compare = other.0.get(k);
            if compare.is_none() {
                return false;
            }
            if !compare.unwrap().eq(v) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Object(pub HashMap<Identifier, Element>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct List(pub Vec<Element>);

pub type Identifier = String;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Value {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    Object(Object),
    List(List),
    FunctionCall(ASTFunctionCall),
    None,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Long(l) => write!(f, "{}", l),
            Value::Float(d) => write!(f, "{}", d),
            Value::Double(d) => write!(f, "{}", d),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Object(o) => write!(f, "{:?}", o),
            Value::List(l) => write!(f, "{:?}", l),
            Value::FunctionCall(fc) => write!(f, "{:?}", fc),
            Value::None => write!(f, "none"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Element {
    FunctionCall(ASTFunctionCall),
    Identifier(Identifier),
    Args(Vec<Element>),
    Value(Value),
    Object(Object),
    List(List),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    None,
}

impl Element {
    pub fn as_string(&self) -> Result<String> {
        match self {
            Element::String(v) => Ok(v.to_string()),
            _ => Err(anyhow!("Unsupported type for as_string {}", self)),
        }
    }

    pub fn to_list(self) -> Result<Vec<Element>> {
        match self {
            Element::List(v) => Ok(v.0),
            _ => Err(anyhow!("Unsupported type for to_list {}", self)),
        }
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::FunctionCall(fc) => write!(f, "{:?}", fc),
            Element::Identifier(i) => write!(f, "{}", i),
            Element::Args(a) => write!(f, "{:?}", a),
            Element::Value(v) => write!(f, "{}", v),
            Element::Object(o) => write!(f, "{:?}", o),
            Element::List(l) => write!(f, "{:?}", l),
            Element::Int(i) => write!(f, "{}", i),
            Element::Long(l) => write!(f, "{}", l),
            Element::Float(d) => write!(f, "{}", d),
            Element::Double(d) => write!(f, "{}", d),
            Element::Bool(b) => write!(f, "{}", b),
            Element::String(s) => write!(f, "{}", s),
            Element::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub elements: Vec<Element>,
}

impl AST {
    pub fn init() -> AST {
        AST { elements: vec![] }
    }

    pub fn merge(&mut self, other: AST) {
        let mut to_append = other.elements;
        self.elements.append(to_append.as_mut())
    }
}

pub fn parse(input: String, config: &ParseConfig) -> Result<AST> {
    let tokens = Tokenizer::parse(Rule::program, input.as_str())?;
    let elements = parse_pairs(tokens, config)?;
    Ok(AST { elements })
}

fn parse_pairs(pairs: Pairs<Rule>, config: &ParseConfig) -> Result<Vec<Element>> {
    let mut results = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::program => results.append(parse_pairs(pair.into_inner(), config)?.as_mut()),
            Rule::function_body => results.append(parse_pairs(pair.into_inner(), config)?.as_mut()),
            Rule::definition => results.append(parse_pairs(pair.into_inner(), config)?.as_mut()),
            Rule::function_call => {
                let inner = parse_pairs(pair.into_inner(), config)?;
                let mut identifier = None;
                let mut definition = None;
                let mut args = Vec::new();
                for element in inner {
                    match element {
                        Element::Identifier(i) => {
                            identifier = Some(i);
                        }
                        Element::Object(o) => {
                            definition = Some(Definition::Object(o));
                        }
                        Element::List(l) => {
                            definition = Some(Definition::List(l));
                        }
                        Element::Args(mut a) => args.append(a.as_mut()),
                        _ => {
                            return Err(anyhow!(
                                "Unexpected Element in `function_call`: {:?}",
                                element
                            ));
                        }
                    }
                }
                results.push(Element::FunctionCall(ASTFunctionCall {
                    identifier: identifier.expect("`identifier` not set for function_call"),
                    args,
                    definition,
                }))
            }
            Rule::identifier => {
                let identifier = pair.as_str().trim();
                results.push(Element::Identifier(identifier.into()));
            }
            Rule::args => results.push(Element::Args(parse_pairs(pair.into_inner(), config)?)),
            Rule::value => {
                let value = parse_pairs(pair.into_inner(), config)?;
                for element in value {
                    let next = match element {
                        Element::Object(object) => Value::Object(object),
                        Element::List(list) => Value::List(list),
                        Element::Int(int) => Value::Int(int),
                        Element::Long(long) => Value::Long(long),
                        Element::Float(float) => Value::Float(float),
                        Element::Double(double) => Value::Double(double),
                        Element::Bool(bool) => Value::Bool(bool),
                        Element::String(string) => Value::String(string),
                        Element::FunctionCall(fc) => Value::FunctionCall(fc),
                        Element::None => Value::None,
                        _ => {
                            return Err(anyhow!("Unexpected Element in `value`: {:?}", element));
                        }
                    };
                    results.push(Element::Value(next));
                }
            }
            Rule::object => {
                let mut definition = HashMap::new();
                let mut last = None;
                for element in parse_pairs(pair.into_inner(), config)? {
                    match element {
                        Element::Identifier(i) => {
                            if last.is_some() {
                                definition.insert(
                                    last.expect("Missing Identifier for Object"),
                                    Element::Identifier(i),
                                );
                                last = None;
                            } else {
                                last = Some(i);
                            }
                        }
                        Element::FunctionCall(f) => {
                            definition.insert(f.identifier.clone(), Element::FunctionCall(f));
                        }
                        Element::Value(v) => {
                            definition.insert(
                                last.expect("Missing Identifier for Object"),
                                Element::Value(v),
                            );
                            last = None;
                        }
                        _ => {
                            return Err(anyhow!("Unexpected Element in `object`: {:?}", element));
                        }
                    }
                }
                results.push(Element::Object(Object(definition)))
            }
            Rule::attribute => results.append(parse_pairs(pair.into_inner(), config)?.as_mut()),
            Rule::list => {
                results.push(Element::List(List(parse_pairs(pair.into_inner(), config)?)))
            }
            Rule::bool => {
                let value = pair.as_str().trim();
                let b = match value {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(anyhow!("Unsupported `bool`: {}", value));
                    }
                };
                results.push(Element::Bool(b));
            }
            Rule::number => {
                let value = pair.as_str().trim();
                let num = if value.contains('.') {
                    if config.use_64_bit_numbers {
                        Element::Float(value.parse()?)
                    } else {
                        Element::Double(value.parse()?)
                    }
                } else if config.use_64_bit_numbers {
                    Element::Long(value.parse()?)
                } else {
                    Element::Int(value.parse()?)
                };
                results.push(num);
            }
            Rule::string => {
                let raw = pair.as_str().trim();
                results.push(Element::String(raw[1..raw.len() - 1].to_string()));
            }
            Rule::VALID_CHARS => {
                return Err(anyhow!(
                    "`VALID_CHARS` called directly, it should be handled in parent"
                ))
            }
            Rule::none => {
                results.push(Element::None);
            }
            Rule::EOI => break,
            Rule::COMMENT => continue,
            Rule::single_line_comment => continue,
            Rule::multi_line_comment => continue,
            Rule::WHITESPACE => continue,
        };
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puts_works() {
        let mut elements = Vec::new();
        let mut args = Vec::new();
        args.push(Element::Value(Value::String("Hello World".to_string())));
        elements.push(Element::FunctionCall(ASTFunctionCall {
            identifier: "puts".into(),
            args,
            definition: None,
        }));

        let result = parse("puts 'Hello World'".to_string(), &ParseConfig::default()).unwrap();
        assert_eq!(result, AST { elements });
    }

    #[test]
    fn let_works() {
        let mut elements = Vec::new();
        let mut details = HashMap::new();
        let accounts = [1, 2, 3]
            .iter()
            .map(|int| Element::Value(Value::Int(*int)))
            .collect();
        details.insert(
            "accounts".into(),
            Element::Value(Value::List(List(accounts))),
        );
        let definition = Some(Definition::Object(Object(details)));
        elements.push(Element::FunctionCall(ASTFunctionCall {
            identifier: "let".into(),
            args: Vec::new(),
            definition,
        }));

        let input = r#"
            let {
                accounts = [1, 2, 3]
            }
        "#
        .to_string();
        let result = parse(input, &ParseConfig::default()).unwrap();
        assert_eq!(result, AST { elements });
    }

    #[test]
    fn symbols_work() {
        let mut elements = Vec::new();
        let mut details = HashMap::new();
        details.insert(
            "account".into(),
            Element::Value(Value::FunctionCall(ASTFunctionCall {
                identifier: ":valid_account".to_string(),
                args: vec![],
                definition: None,
            })),
        );
        let definition = Some(Definition::Object(Object(details)));
        elements.push(Element::FunctionCall(ASTFunctionCall {
            identifier: ":allow".to_string(),
            args: Vec::new(),
            definition,
        }));

        let input = r#"
            :allow {
                account = :valid_account
            }
        "#
        .to_string();
        let result = parse(input, &ParseConfig::default()).unwrap();
        assert_eq!(result, AST { elements });
    }

    #[test]
    fn function_call_in_object_allowed() {
        let mut elements = Vec::new();
        let mut details = HashMap::new();
        let mut inner_details = HashMap::new();
        let accounts = [1, 2, 3]
            .iter()
            .map(|int| Element::Value(Value::Int(*int)))
            .collect();
        inner_details.insert(
            "account".into(),
            Element::Value(Value::FunctionCall(ASTFunctionCall {
                identifier: "one_of".into(),
                args: Vec::new(),
                definition: Some(Definition::List(List(accounts))),
            })),
        );
        details.insert(
            "variables".into(),
            Element::FunctionCall(ASTFunctionCall {
                identifier: "variables".into(),
                args: vec![],
                definition: Some(Definition::Object(Object(inner_details))),
            }),
        );
        let definition = Some(Definition::Object(Object(details)));
        elements.push(Element::FunctionCall(ASTFunctionCall {
            identifier: "allow".into(),
            args: Vec::new(),
            definition,
        }));

        let input = r#"
            allow {
              variables {
                account = one_of [1, 2, 3]
              }
            }
        "#
        .to_string();
        let result = parse(input, &ParseConfig::default()).unwrap();
        assert_eq!(result, AST { elements });
    }
}
