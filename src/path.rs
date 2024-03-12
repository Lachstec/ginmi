use crate::gen::gnmi::{Path as GnmiPath, PathElem};
use std::collections::HashMap;
use std::convert::From;

#[derive(Debug, Clone)]
struct PathElement {
    name: String,
    values: HashMap<String, String>,
}

impl PathElement {
    pub fn new(name: &str, values: HashMap<String, String>) -> Self {
        Self {
            name: String::from(name),
            values,
        }
    }
}

impl From<PathElement> for PathElem {
    fn from(elem: PathElement) -> Self {
        Self {
            name: elem.name,
            key: elem.values,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    target: String,
    origin: String,
    elements: Vec<PathElement>,
}

impl Path {
    pub fn new(target: &str, origin: &str) -> Self {
        Self {
            target: String::from(target),
            origin: String::from(origin),
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, name: &str, values: HashMap<String, String>) {
        self.elements.push(PathElement::new(name, values))
    }
}

impl From<Path> for GnmiPath {
    fn from(p: Path) -> Self {
        Self {
            origin: p.origin,
            elem: p
                .elements
                .into_iter()
                .map(|elem| PathElem::from(elem))
                .collect::<Vec<PathElem>>(),
            target: p.target,
            ..Default::default()
        }
    }
}
