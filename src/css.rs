#[derive(Debug)]
pub struct StyleSheet {
    rules: Vec<Rule>,
}

#[derive(Debug)]
struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

#[derive(Debug)]
enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug)]
struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>,
}

#[derive(Debug)]
struct Declaration {
    name: String,
    value: Value,
}

#[derive(Debug)]
enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug)]
enum Unit {
    Px,
}

#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    pub fn parse(source: String) -> StyleSheet {
        let mut parser = Parser {
            pos: 0,
            input: source,
        };
        StyleSheet {
            rules: parser.parse_rules(),
        }
    }

    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = vec![];
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }

            rules.push(self.parse_rule());
        }

        rules
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));

        self.pos += next_pos;
        cur_char
    }

    fn consume_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut res = String::new();

        while !self.eof() && predicate(self.next_char()) {
            res.push(self.consume_char())
        }

        res
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    fn consume_whitespace(&mut self) -> String {
        self.consume_while(char::is_whitespace)
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: vec![],
        };

        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();

                    let identifier = self.parse_identifier();
                    selector.class.push(identifier);
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }

        selector
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = vec![];
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();

            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break, // start of declarations
                c => panic!("Unexpected character {} in selector list", c),
            }
        }

        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');

        let mut declarations = vec![];
        loop {
            self.consume_whitespace();

            if self.next_char() == '}' {
                self.consume_char();
                break;
            }

            declarations.push(self.parse_declaration());
        }

        declarations
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();

        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();

        let value = self.parse_value();

        assert_eq!(self.consume_char(), ';');

        Declaration { name, value }
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| match c {
            '0'..='9' | '.' => true,
            _ => false,
        });

        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("Unrecognised unit"),
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');

        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];

        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
        _ => false,
    }
}

// region Specificity
pub type Specificity = (usize, usize, usize);

impl Selector {
    /// Selectors have specificity based on IDs, classes and tag counts.
    /// IDs have highest priority, then classes, then tags
    /// Returns a tuple of (ID count, class count, tag count)
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;

        let id_count = simple.id.iter().count();
        let class_count = simple.class.len();
        let tag_count = simple.tag_name.iter().count();

        (id_count, class_count, tag_count)
    }
}
