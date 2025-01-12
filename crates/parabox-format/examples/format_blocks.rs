#![allow(dead_code)]

use anstyle::{AnsiColor, RgbColor, Style};
use parabox::{BlockKey, World};
use parabox_format::{Formatter, MetaFmt};
use parabox_parser::{Executor, MetaTable, StringSource};
use std::rc::Rc;

const SCRIPT: &str = r#"
DEFINE BOX #container size (5, 5)
DEFINE WALL #wall
DEFINE BOX #box size (3, 3)
DEFINE ALIAS #alias ref #box
DEFINE INFINITY #infinity ref #box
DEFINE EPSILON #epsilon ref #box size (3, 3)

PLACE #wall at (0, 0) in #container
PLACE #box at (1, 1) in #container
PLACE #alias at (2, 2) in #container
PLACE #infinity at (3, 3) in #container
PLACE #epsilon at (4, 4) in #container
"#;

struct Wrapper<'a>(&'a World, &'a MetaTable);

impl MetaFmt for Wrapper<'_> {
    fn fmt_repr(&self, key: BlockKey) -> char {
        match self.1.get_name(&key).unwrap() {
            s if s == "container" => '0',
            s if s == "wall" => '#',
            s if s == "box" => 'p',
            s if s == "alias" => 'p',
            s if s == "infinity" => 'p',
            s if s == "epsilon" => 'p',
            _ => unreachable!("unknown block key"),
        }
    }

    fn fmt_style(&self, key: BlockKey) -> Style {
        match self.1.get_name(&key).unwrap() {
            s if s == "container" => AnsiColor::Black.on_default(),
            s if s == "wall" => AnsiColor::Black.on_default(),
            s if s == "box" => AnsiColor::Red.on_default(),
            s if s == "alias" => AnsiColor::BrightRed.on_default().italic(),
            s if s == "infinity" => AnsiColor::BrightRed.on_default().bold(),
            s if s == "epsilon" => AnsiColor::BrightRed.on_default().italic().bold(),
            _ => unreachable!("unknown block key"),
        }
    }
}

fn main() {
    let mut executor = Executor::new();
    let source = StringSource::new(SCRIPT.to_string());
    executor.push_source(Rc::new(source)).unwrap();
    executor.run_all().unwrap();
    let (world, meta) = executor.take();

    let wrapper = Wrapper(&world, &meta);
    let background = Some(RgbColor(216, 216, 216).into());
    let formatter = Formatter::new(&world, &wrapper).with_background(background);

    println!("{}", formatter.format(2).render());
}
