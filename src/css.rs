#[derive(Debug)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Clone, Debug)]
pub enum Unit {
    Px,
    Percent
}

#[derive(Clone, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct Parser {
    pub pos: usize,
    pub input: String,
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

    fn next_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
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

        while !self.eof() && predicate(self.next_char().unwrap()) {
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
                Some('#') => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                Some('.') => {
                    self.consume_char();

                    let identifier = self.parse_identifier();
                    selector.class.push(identifier);
                }
                Some('*') => {
                    self.consume_char();
                }
                Some(c) if valid_identifier_char(c) => {
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
                Some(',') => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                Some('{') => {
                    // start of declarations - break. Consume char for use in inline styles (which do not have brackets).
                    self.consume_char();
                    break
                }, 
                c => panic!("Unexpected character {:?} in selector list", c),
            }
        }

        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    pub fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = vec![];
        loop {
            self.consume_whitespace();

            if self.next_char() == Some('}') {
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
            Some('0'..='9') => self.parse_length(),
            Some('#') => self.parse_color(),
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
            "%" => Unit::Percent,
            c => panic!("Unrecognised unit: {}", c),
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
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '%' => true,
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
