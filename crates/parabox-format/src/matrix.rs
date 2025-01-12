use anstyle::Style;
use std::ops::Add;

#[derive(Copy, Clone)]
pub struct Cell(char, Style);

impl Cell {
    pub fn new(repr: char, style: Style) -> Self {
        Self(repr, style)
    }

    pub fn default(repr: char) -> Self {
        Self(repr, Style::default())
    }

    pub fn repr(&self) -> char {
        self.0
    }

    pub fn style(&self) -> Style {
        self.1
    }
}

#[derive(Clone)]
pub struct Matrix {
    /// First dimension is the y-axis, second dimension is the x-axis.
    matrix: Vec<Vec<Cell>>,
    size: (usize, usize),
}

impl Matrix {
    pub fn new(width: usize, height: usize) -> Self {
        let matrix = vec![vec![Cell(' ', Style::default()); width]; height];

        Self {
            matrix,
            size: (width, height),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.matrix[y][x] = cell;
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.matrix[y][x]
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn width(&self) -> usize {
        self.size.0
    }

    pub fn height(&self) -> usize {
        self.size.1
    }

    pub fn resize(&mut self, size: (usize, usize)) {
        self.matrix
            .resize(size.1, vec![Cell(' ', Style::default()); size.0]);

        for row in &mut self.matrix {
            row.resize(size.0, Cell(' ', Style::default()));
        }
    }

    pub fn render(&self) -> String {
        let (width, height) = self.size;
        let mut output = String::new();

        for y in 0..height {
            for x in 0..width {
                let Cell(repr, style) = &self.matrix[y][x];
                output.push_str(&format!("{style}{repr}{style:#}"));
            }

            output.push('\n');
        }

        output
    }
}

impl Add for Matrix {
    type Output = Self;

    fn add(mut self, mut other: Self) -> Self {
        let height = usize::max(self.size.1, other.size.1);
        self.resize((self.size.0, height));
        other.resize((other.size.0, height));

        for y in 0..height {
            self.matrix[y].append(&mut other.matrix[y]);
        }

        self.size = (self.size.0 + other.size.0, height);

        self
    }
}

pub struct MatrixBuilder {
    matrix: Vec<Vec<Cell>>,
    width: usize,
}

impl MatrixBuilder {
    pub fn new() -> Self {
        Self {
            matrix: Vec::new(),
            width: 0,
        }
    }

    pub fn push_space(&mut self) {
        self.push_cell(Cell(' ', Style::default()));
    }

    pub fn push_str(&mut self, string: &str) {
        for c in string.chars() {
            self.push_cell(Cell(c, Style::default()));
        }
    }

    pub fn push_cell(&mut self, cell: Cell) {
        if self.matrix.is_empty() {
            self.push_newline();
        }
        let last = self.matrix.last_mut().unwrap();
        last.push(cell);
        self.width = last.len().max(self.width);
    }

    pub fn push_newline(&mut self) {
        self.matrix.push(Vec::new());
    }

    pub fn build(self) -> Matrix {
        let completed = self
            .matrix
            .into_iter()
            .map(|mut row| {
                row.resize(self.width, Cell(' ', Style::default()));
                row
            })
            .collect::<Vec<_>>();

        let height = completed.len();

        Matrix {
            matrix: completed,
            size: (self.width, height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_render() {
        let mut matrix = Matrix::new(3, 3);
        matrix.set(0, 0, Cell::default('a'));
        matrix.set(1, 1, Cell::default('b'));
        matrix.set(2, 2, Cell::default('c'));

        let expected = "a  \n b \n  c\n";
        assert_eq!(matrix.render(), expected);
    }

    #[test]
    fn test_matrix_add() {
        let mut matrix1 = Matrix::new(3, 3);
        matrix1.set(0, 0, Cell::new('a', Style::default()));
        matrix1.set(1, 1, Cell::new('b', Style::default()));
        matrix1.set(2, 2, Cell::new('c', Style::default()));

        let mut matrix2 = Matrix::new(3, 3);
        matrix2.set(0, 0, Cell::new('d', Style::default()));
        matrix2.set(1, 1, Cell::new('e', Style::default()));
        matrix2.set(2, 2, Cell::new('f', Style::default()));

        let expected = "a  d  \n b  e \n  c  f\n";
        assert_eq!((matrix1 + matrix2).render(), expected);
    }

    #[test]
    fn test_matrix_builder() {
        let mut builder = MatrixBuilder::new();
        builder.push_str("abc");
        builder.push_newline();
        builder.push_str("def");
        builder.push_newline();
        builder.push_str("ghi");

        let matrix = builder.build();
        let expected = "abc\n\
                        def\n\
                        ghi\n";
        assert_eq!(matrix.render(), expected);
    }
}
