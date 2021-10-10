pub mod css;
pub mod dom;

use css::Parser as CSSParser;
use dom::Parser as DomParser;

fn main() {
    let html_res = DomParser::parse("<html><body>Hello, world!</body></html>".to_string());
    println!("{:?}", html_res);
    println!("\n");

    let css_res = CSSParser::parse("
      .silly { background-color: powderblue; width: 100%; }
      .billy { color: #0000FF; }
      .boo { color: #FF0000; }
    ".to_string());

    let multi_css_res = CSSParser::parse("
      .silly, .funny { background-color: slateblue; }
    ".to_string());

    println!("{:?}", multi_css_res);
}
