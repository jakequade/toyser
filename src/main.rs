pub mod css;
pub mod dom;
pub mod style;

use css::Parser as CSSParser;
use dom::Parser as DomParser;

fn main() {
  let html_res = DomParser::parse("<html><body><p class=\"silly\">Hello, world!</p><p style=\"name:inline-style;color: #FF0000;\">How are we?</p></body></html>".to_string());
  println!("{:?}", html_res);
  println!("\n");

  let css_res = CSSParser::parse(
    "
      .silly { background-color: transparent; width: 100%; color: #0000FF; }
      .billy { color: #0000FF; }
      .boo { color: #FF0000; }
    "
    .to_string(),
  );

  println!("{:?}", css_res);
  println!("\n");

  let tree = style::style_tree(&html_res, &css_res);
  println!("{:#?}", tree);
}
