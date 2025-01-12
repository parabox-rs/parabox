use crate::matrix::{Cell, Matrix, MatrixBuilder};
use anstyle::{Color, Style};
use parabox::{BlockKey, World};
use slotmap::SlotMap;
use std::collections::HashMap;

pub trait MetaFmt {
    fn fmt_repr(&self, key: BlockKey) -> char;

    fn fmt_style(&self, key: BlockKey) -> Style;
}

impl MetaFmt for HashMap<BlockKey, Cell> {
    fn fmt_repr(&self, key: BlockKey) -> char {
        self.get(&key).map(|cell| cell.repr()).unwrap_or(' ')
    }

    fn fmt_style(&self, key: BlockKey) -> Style {
        self.get(&key).map(|cell| cell.style()).unwrap_or_default()
    }
}

impl MetaFmt for SlotMap<BlockKey, Cell> {
    fn fmt_repr(&self, key: BlockKey) -> char {
        self.get(key).map(|cell| cell.repr()).unwrap_or(' ')
    }

    fn fmt_style(&self, key: BlockKey) -> Style {
        self.get(key).map(|cell| cell.style()).unwrap_or_default()
    }
}

impl MetaFmt for HashMap<BlockKey, char> {
    fn fmt_repr(&self, key: BlockKey) -> char {
        self.get(&key).copied().unwrap_or(' ')
    }

    fn fmt_style(&self, _key: BlockKey) -> Style {
        Style::default()
    }
}

impl MetaFmt for SlotMap<BlockKey, char> {
    fn fmt_repr(&self, key: BlockKey) -> char {
        self.get(key).copied().unwrap_or(' ')
    }

    fn fmt_style(&self, _key: BlockKey) -> Style {
        Style::default()
    }
}

pub struct Formatter<'a, S>
where
    S: MetaFmt,
{
    world: &'a World,
    meta_style: &'a S,
    background: Option<Color>,
}

impl<'a, S> Formatter<'a, S>
where
    S: MetaFmt + 'a,
{
    pub fn new(world: &'a World, meta_style: &'a S) -> Self {
        Self {
            world,
            meta_style,
            background: None,
        }
    }

    pub fn with_background(mut self, background: Option<Color>) -> Self {
        self.background = background;
        self
    }

    pub fn format_cell(&self, key: BlockKey) -> Cell {
        let repr = self.meta_style.fmt_repr(key);
        let style = self.meta_style.fmt_style(key);

        Cell::new(repr, style.bg_color(self.background))
    }

    pub fn format_block(&self, key: BlockKey) -> Matrix {
        let mut builder = MatrixBuilder::new();

        let block = &self.world[key];
        let interior = &block.state.interior;
        let (width, height) = block.proto.size();

        for y in (0..height).rev() {
            builder.push_newline();

            if y == height / 2 {
                builder.push_cell(self.format_cell(key));
                builder.push_space();
            } else {
                builder.push_space();
                builder.push_space();
            }

            for x in 0..width {
                if let Some(cell) = interior[x][y] {
                    builder.push_cell(self.format_cell(cell));
                } else {
                    builder.push_cell(Cell::new(' ', Style::new().bg_color(self.background)));
                }
            }
        }

        builder.build()
    }

    pub fn format(&self, space_between: usize) -> Matrix {
        self.world
            .blocks()
            .iter()
            .filter(|(_, block)| block.proto.is_hollow())
            .map(|(key, _)| self.format_block(key))
            .reduce(|matrix1, matrix2| matrix1 + Matrix::new(space_between, 0) + matrix2)
            .unwrap_or_else(|| Matrix::new(0, 0))
    }
}
