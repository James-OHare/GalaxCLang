// Source span and location tracking.
// Every token and AST node carries a Span so that diagnostics can point
// back to the exact source position of an error.

use serde::Serialize;

/// Byte-offset span within a source file. Both endpoints are inclusive of start,
/// exclusive of end, matching the convention used by most editor tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Span {
    /// Byte offset of the first character.
    pub start: usize,
    /// Byte offset one past the last character.
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "span start must not exceed end");
        Span { start, end }
    }

    /// A zero-width span at a single position, used for synthetic tokens.
    pub fn point(offset: usize) -> Self {
        Span { start: offset, end: offset }
    }

    /// Merge two spans into one that covers both.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extract the slice of source text covered by this span.
    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

/// Resolved line/column location for display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub filename: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourceLocation {
    /// Resolve a byte offset into a line/column pair by scanning the source.
    pub fn from_offset(source: &str, offset: usize, filename: &str) -> Self {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        SourceLocation {
            filename: filename.to_string(),
            line,
            column: col,
            offset,
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_merge() {
        let a = Span::new(5, 10);
        let b = Span::new(8, 15);
        let merged = a.merge(b);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn source_location_single_line() {
        let src = "let x = 42";
        let loc = SourceLocation::from_offset(src, 4, "test.gxc");
        assert_eq!(loc.line, 1);
        assert_eq!(loc.column, 5);
    }

    #[test]
    fn source_location_multiline() {
        let src = "line one\nline two\nline three";
        let loc = SourceLocation::from_offset(src, 14, "test.gxc");
        assert_eq!(loc.line, 2);
        assert_eq!(loc.column, 6);
    }
}
