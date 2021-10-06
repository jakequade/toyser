use std::collections::HashMap;

#[derive(Debug)]
pub struct Node {
    children: Vec<Node>,
    node_type: NodeType,
}

#[derive(Debug)]
enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug)]
struct ElementData {
    tag_name: String,
    attributes: AttrMap,
}

type AttrMap = HashMap<String, String>;

fn text(data: String) -> Node {
    Node {
        children: vec![],
        node_type: NodeType::Text(data),
    }
}

fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
    }
}

// region: parser

pub struct Parser {
    pos: usize, // index of next, unprocessed character
    input: String,
}

impl Parser {
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));

        self.pos += next_pos;
        cur_char
    }

    fn consume_while<F>(&mut self, pred: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();

        while !self.eof() && pred(self.next_char()) {
            result.push(self.consume_char());
        }

        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
    }

    // region parsing nodes
    fn parse_node(&mut self) -> Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    fn parse_text(&mut self) -> Node {
        text(self.consume_while(|c| c != '<'))
    }

    fn parse_element(&mut self) -> Node {
        // opening tag (e.g. '<xml type="url">')
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attrs();

        assert!(self.consume_char() == '>');

        // contents
        let children = self.parse_nodes();

        // closing tag (e.g. '</xml>)
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        return elem(tag_name, attrs, children);
    }

    /// Parse attribute key/value pair
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        return (name, value);
    }

    /// Parse attribute value, in quotes
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');

        let value = self.consume_while(|c| c != open_quote);

        // closing quote
        assert!(self.consume_char() == open_quote);
        value
    }

    /// Parse all key/value attribute pairs for a given element
    fn parse_attrs(&mut self) -> AttrMap {
        let mut attributes = HashMap::new();

        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }

            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }

        attributes
    }

    /// recursively call parse_node() with consume_whitespace() until we reach the
    /// corresponding closing tag
    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = vec![];

        loop {
            self.consume_whitespace();

            if self.eof() || self.starts_with("</") {
                break;
            }

            nodes.push(self.parse_node());
        }

        nodes
    }

    /// Parse an entire HTML document and return the root element
    pub fn parse(source: String) -> Node {
        let mut nodes = Parser {
            pos: 0,
            input: source,
        }
        .parse_nodes();

        // if the document contains a root element, just return it. Otherwise, create one.
        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            elem("html".to_string(), HashMap::new(), nodes)
        }
    }
}
