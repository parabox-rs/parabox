use crate::lexer::is_newline_char;
use ecow::EcoString;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;
use std::path::PathBuf;
use unscanny::Scanner;

/// A trait for sources of text.
///
/// Parser deals with a single line of command. It is the responsibility of the
/// source to split a text into lines and provide the location information.
pub trait Source {
    /// Returns the name of the source.
    fn name(&self) -> EcoString;

    /// Returns the text of the source.
    fn text(&self) -> &str;

    /// Returns the line at the given index.
    fn line(&self, line: usize) -> Option<&str>;

    /// Returns the number of lines in the source.
    fn line_len(&self) -> usize;

    /// Returns the lines of the source.
    fn lines(&self) -> Vec<&str>;

    /// Returns the range of the line at the given index.
    fn line_range(&self, line: usize) -> Option<Range<usize>>;

    /// Returns the line and column of the cursor.
    fn locate(&self, cursor: usize) -> (usize, usize);
}

/// A anonymous source of text from a string.
pub struct StringSource {
    text: String,
    ranges: Vec<Range<usize>>,
}

impl StringSource {
    /// Creates a new string source from a text.
    pub fn new(text: String) -> Self {
        let mut ranges = vec![];
        let mut s = Scanner::new(&text);

        let mut start = 0;

        while let Some(c) = s.eat() {
            if is_newline_char(c) {
                ranges.push(start..(s.cursor() - 1));

                if c == '\r' {
                    s.eat_if('\n');
                }

                start = s.cursor();
            }
        }

        if start < s.cursor() {
            ranges.push(start..s.cursor());
        }

        Self { text, ranges }
    }
}

impl Source for StringSource {
    fn name(&self) -> EcoString {
        let hasher = &mut DefaultHasher::new();
        self.text.hash(hasher);
        format!("<string#{:x}>", hasher.finish()).into()
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn line(&self, line: usize) -> Option<&str> {
        self.ranges.get(line).map(|range| &self.text[range.clone()])
    }

    fn line_len(&self) -> usize {
        self.ranges.len()
    }

    fn lines(&self) -> Vec<&str> {
        let text = self.text();
        let mut lines = vec![];

        for range in &self.ranges {
            lines.push(&text[range.clone()]);
        }

        lines
    }

    fn line_range(&self, line: usize) -> Option<Range<usize>> {
        self.ranges.get(line).map(|range| range.clone())
    }

    fn locate(&self, cursor: usize) -> (usize, usize) {
        let mut line = 0;

        for range in &self.ranges {
            if cursor < range.end {
                return (line, cursor - range.start);
            }

            line += 1;
        }

        (line, cursor - self.ranges.last().unwrap().start)
    }
}

/// A named source of text from a string.
pub struct NamedStringSource {
    name: EcoString,
    string_source: StringSource,
}

impl NamedStringSource {
    /// Creates a new named string source from a name and a text.
    pub fn new(name: EcoString, text: String) -> Self {
        let string_source = StringSource::new(text);

        Self {
            name,
            string_source,
        }
    }
}

impl Source for NamedStringSource {
    fn name(&self) -> EcoString {
        self.name.clone()
    }

    fn text(&self) -> &str {
        self.string_source.text()
    }

    fn line(&self, line: usize) -> Option<&str> {
        self.string_source.line(line)
    }

    fn line_len(&self) -> usize {
        self.string_source.line_len()
    }

    fn lines(&self) -> Vec<&str> {
        self.string_source.lines()
    }

    fn line_range(&self, line: usize) -> Option<Range<usize>> {
        self.string_source.line_range(line)
    }

    fn locate(&self, cursor: usize) -> (usize, usize) {
        self.string_source.locate(cursor)
    }
}

/// A source of text from a file.
pub struct FileSource {
    path: PathBuf,
    name: EcoString,
    string_source: StringSource,
}

impl FileSource {
    /// Opens a file source from a path.
    pub fn open(path: PathBuf) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(&path)?;
        let name = path.file_name().unwrap().to_string_lossy().into();
        let string_source = StringSource::new(text);

        Ok(Self {
            path,
            name,
            string_source,
        })
    }
}

impl Source for FileSource {
    fn name(&self) -> EcoString {
        self.name.clone()
    }

    fn text(&self) -> &str {
        self.string_source.text()
    }

    fn line(&self, line: usize) -> Option<&str> {
        self.string_source.line(line)
    }

    fn line_len(&self) -> usize {
        self.string_source.line_len()
    }

    fn lines(&self) -> Vec<&str> {
        self.string_source.lines()
    }

    fn line_range(&self, line: usize) -> Option<Range<usize>> {
        self.string_source.line_range(line)
    }

    fn locate(&self, cursor: usize) -> (usize, usize) {
        self.string_source.locate(cursor)
    }
}
