extern crate core;

use anstyle::{RgbColor, Style};
use parabox::{BlockKey, ProtoType, World};
use parabox_format::{Formatter, MetaFmt};
use parabox_parser::{Executor, MetaTable, ParseResult, StringSource};
use std::io::Write;
use std::rc::Rc;

fn brighten(color: RgbColor) -> RgbColor {
    let factor = 1.5;
    let r = (color.0 as f32 * factor).min(255.0) as u8;
    let g = (color.1 as f32 * factor).min(255.0) as u8;
    let b = (color.2 as f32 * factor).min(255.0) as u8;

    RgbColor(r, g, b)
}

struct Wrapper {
    executor: Executor,
    names: Vec<BlockKey>,
}

impl Wrapper {
    fn new() -> Self {
        Self {
            executor: Executor::new(),
            names: Vec::new(),
        }
    }

    fn execute(&mut self, text: &str) -> ParseResult<()> {
        let source = StringSource::new(text.to_string());
        self.executor.push_source(Rc::new(source))?;
        self.executor.run_all()?;

        for (key, block) in self.executor.world().blocks() {
            if block.proto.is_hollow() {
                if !self.names.contains(&key) {
                    self.names.push(key);
                }
            }
        }

        assert!(self.names.len() < 10, "too many blocks");

        Ok(())
    }

    fn world(&self) -> &World {
        self.executor.world()
    }

    fn meta(&self) -> &MetaTable {
        self.executor.meta()
    }
}

impl MetaFmt for Wrapper {
    fn fmt_repr(&self, mut key: BlockKey) -> char {
        let block = &self.world()[key];

        if let Some(reference) = block.proto.reference() {
            key = reference;
        }

        match self.names.iter().position(|&k| k == key) {
            Some(i) => std::char::from_digit(i as u32, 10).unwrap(),
            None => '#', // Wall
        }
    }

    fn fmt_style(&self, key: BlockKey) -> Style {
        let block = &self.world()[key];
        let repr = self.fmt_repr(key);

        let color = match repr {
            '0' => RgbColor(128, 0, 0),   // Red
            '1' => RgbColor(0, 128, 0),   // Green
            '2' => RgbColor(0, 0, 128),   // Blue
            '3' => RgbColor(128, 128, 0), // Yellow
            '4' => RgbColor(0, 128, 128), // Cyan
            '5' => RgbColor(128, 0, 128), // Magenta
            '6' => RgbColor(96, 96, 96),  // Silver
            '7' => RgbColor(64, 64, 64),  // Gray
            '8' => RgbColor(64, 0, 0),    // Maroon
            '9' => RgbColor(64, 64, 0),   // Olive
            _ => RgbColor(0, 0, 0),       // Black
        };

        let style = if block.proto.reference().is_some() {
            brighten(color)
        } else {
            color
        }
        .on_default();

        match block.proto {
            ProtoType::Alias { .. } => style.italic(),
            ProtoType::Infinity { .. } => style.italic().bold(),
            ProtoType::Epsilon { .. } => style.italic().bold().underline(),
            _ => style,
        }
    }
}

impl Wrapper {
    fn format(&self) -> String {
        let formatter = Formatter::new(self.executor.world(), self)
            .with_background(Some(RgbColor(232, 232, 232).into()));

        let mut buffer = String::new();

        for (i, key) in self.names.iter().enumerate() {
            let name = self.meta().get_name(key).unwrap();
            buffer += &format!("  {i}: {name}");
        }

        buffer += "\n\n";
        buffer += formatter.format(4).render().as_str();
        buffer += "\n";

        buffer
    }
}

fn main() {
    let mut wrapper = Wrapper::new();

    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" {
            break;
        }

        match wrapper.execute(input) {
            Ok(_) => println!("\n{}", wrapper.format()),
            Err(e) => eprintln!("{}", e),
        }
    }
}
