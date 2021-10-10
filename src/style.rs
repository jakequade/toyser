/// This module takes CSS rules and applies them to suitable dom elements

use crate::{ css::{Parser as CssParser, Rule, Selector, SimpleSelector, Specificity, StyleSheet, Value}, dom::{ ElementData, Node, NodeType}};
use std::collections::HashMap;

type PropertyMap = HashMap<String, Value>;

#[derive(Debug)]
pub struct StyledNode<'a> {
    children: Vec<StyledNode<'a>>,
    node: &'a Node,
    specified_values: PropertyMap,
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple) => matches_simple_selector(elem, simple),
        // _ => {
        //     false
        // }
    }
}

/// Searches an elements properties to find unmatching values,
/// otherwise returns true
fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check ID selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    // TODO: this should allow some non-matching classes, as an element can have multiple classes
    // and only apply css from some
    if selector.class.iter().any(|class| !elem.classes().contains(&**class)) {
        return false;
    }

    // No-non-matching selectors found, apply styles
    true
}

fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    // find highest-specificity (first) matching selector.
    rule.selectors.iter()
    .find(|selector| matches(elem, *selector))
    .map(|selector| (selector.specificity(), rule))
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a StyleSheet) -> Vec<MatchedRule<'a>> {
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

/// Apply styles to a single element, returning the specified values
fn specified_values(elem: &ElementData, stylesheet: &StyleSheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    // sort by Specificity
    rules.sort_by(|&(a, _), (b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    match elem.attributes.get("style") {
        Some(inline_styles) => {
            let mut inline_parser = CssParser { pos: 0, input: inline_styles.to_string() };
            let declarations = inline_parser.parse_declarations();

            for declaration in declarations {
                values.insert(declaration.name, declaration.value);
            }
        },
        None => ()
    }

    values
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a StyleSheet) -> StyledNode<'a> {
    StyledNode {
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect(),
        node: root,
        specified_values: match &root.node_type {
            NodeType::Text(_) => HashMap::new(),
            NodeType::Element(data) => specified_values(data, stylesheet)
        }
    }
}