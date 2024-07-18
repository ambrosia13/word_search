#![warn(missing_docs)]

//! # Word searches
//!
//! A crate that helps with generating word searches with flexible configuration options.

use std::{collections::HashSet, fmt::Display, ops::Index};

use array2d::Array2D;
use rand::Rng;

/// An error that happened when creating the word search.
#[derive(Clone, Copy, Debug)]
pub enum Error<'a> {
    /// Either the number of rows or the number of columns in the word search config is too small, meaning that
    /// not all words in the given list can fit in the grid.
    DimensionsTooSmall(usize, usize, &'a [String]),

    /// When the word search was configured to fill non-word spaces using only letters contained in the word, but
    /// no words were given when creating the word search, this error is returned.
    NoGivenLettersToUseInGrid,
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DimensionsTooSmall(num_rows, num_columns, words) => {
                write!(
                    f,
                    "Grid dimensions {} rows x {} columns is too small for word list {:?}",
                    num_rows, num_columns, words
                )
            }
            Error::NoGivenLettersToUseInGrid => {
                write!(f, "Word search was configured to only use the letters from the given word list to fill the grid, but no words were provided")
            }
        }
    }
}

impl<'a> std::error::Error for Error<'a> {}

/// The direction a word is placed in inside the word search grid.
#[derive(Clone, Copy, Debug)]
pub enum WordDirection {
    /// The word goes up from the start position.
    Up,

    /// The word goes down from the start position.
    Down,

    /// The word goes left from the start position.
    Left,

    /// The word goes right from the start position.
    Right,

    /// The word goes diagonally up and left from the start position.
    DiagonalUpLeft,

    /// The word goes diagonally up and right from the start position.
    DiagonalUpRight,

    /// The word goes diagonally down and left from the start position.
    DiagonalDownLeft,

    /// The word goes diagonally down and right from the start position.
    DiagonalDownRight,
}

impl WordDirection {
    /// Returns a random word direction.
    pub fn random() -> Self {
        use WordDirection::*;

        let n = rand::thread_rng().gen_range(0..8);

        match n {
            0 => Up,
            1 => Down,
            2 => Left,
            3 => Right,
            4 => DiagonalUpLeft,
            5 => DiagonalUpRight,
            6 => DiagonalDownLeft,
            7 => DiagonalDownRight,
            _ => unreachable!(),
        }
    }

    /// Returns a random "forward-facing" direction (e.g. excluding [WordDirection::Up] and all left-facing directions)
    pub fn random_forward() -> Self {
        use WordDirection::*;

        let n = rand::thread_rng().gen_range(0..4);

        match n {
            0 => Down,
            1 => Right,
            2 => DiagonalUpRight,
            3 => DiagonalDownRight,
            _ => unreachable!(),
        }
    }
}

/// Describes where a word's letters are placed in the word search grid. Includes a beginning coordinate, a length, and a direction.
#[derive(Debug)]
pub struct WordSpan {
    /// The starting coordinate in the grid of the word that this WordSpan refers to.
    pub begin: (usize, usize),

    /// The length of the word that this WordSpan refers to.
    pub len: usize,

    /// The direction that the word goes in.
    pub direction: WordDirection,
}

impl WordSpan {
    /// Creates a new [WordSpan] with the given values for the beginning coordinate, the length, and the direction of the word.
    pub fn new(begin: (usize, usize), len: usize, direction: WordDirection) -> Self {
        Self {
            begin,
            len,
            direction,
        }
    }

    /// Returns all indices of the grid that the word spans across.
    pub fn indices(&self) -> Vec<(usize, usize)> {
        use WordDirection::*;

        let mut indices = Vec::with_capacity(self.len);

        for i in 0..self.len {
            let mut index = self.begin;

            match self.direction {
                Up => index.1 += i,
                Down => index.1 -= i,
                Left => index.0 -= i,
                Right => index.0 += i,
                DiagonalUpLeft => {
                    index.0 -= i;
                    index.1 += i;
                }
                DiagonalUpRight => {
                    index.0 += i;
                    index.1 += i;
                }
                DiagonalDownLeft => {
                    index.0 -= i;
                    index.1 -= i;
                }
                DiagonalDownRight => {
                    index.0 += i;
                    index.1 -= i;
                }
            }

            indices.push(index);
        }

        indices
    }

    /// Returns whether two word spans overlap, meaning that they can't both be placed on the grid.
    pub fn overlaps(&self, other: &Self) -> bool {
        let other_indices = other.indices();

        !self
            .indices()
            .iter()
            .any(|index| other_indices.contains(index))
    }

    fn get_end_coordinate(&self) -> (isize, isize) {
        use WordDirection::*;

        let mut end = (self.begin.0 as isize, self.begin.1 as isize);
        let len = self.len as isize;

        match self.direction {
            Up => end.1 += len,
            Down => end.1 -= len,
            Left => end.0 -= len,
            Right => end.0 += len,
            DiagonalUpLeft => {
                end.0 -= len;
                end.1 += len;
            }
            DiagonalUpRight => {
                end.0 += len;
                end.1 += len;
            }
            DiagonalDownLeft => {
                end.0 -= len;
                end.1 -= len;
            }
            DiagonalDownRight => {
                end.0 += len;
                end.1 -= len;
            }
        }

        end
    }

    /// Returns whether the word span is in bounds of the given grid dimensions.
    pub fn in_bounds(&self, num_rows: usize, num_columns: usize) -> bool {
        let end = self.get_end_coordinate();

        // Test that both the beginning and ending coordinates are in the grid
        self.begin.0 < num_rows
            && self.begin.1 < num_columns
            && end.0.is_positive()
            && end.1.is_positive()
            && (end.0 as usize) < num_rows
            && (end.1 as usize) < num_columns
    }
}

/// The configuration for the word search. See [`WordSearch::new`] for details.
///
/// [`WordSearch::new`]: struct.WordSearch.html#method.new
#[derive(Debug)]
pub struct WordSearchConfig<'a> {
    /// The number of rows.
    pub num_rows: usize,

    /// The number of columns.
    pub num_columns: usize,

    /// The list of words that will appear in the word search.
    pub words: &'a [String],

    /// Whether to fill empty (non-word) spaces in the word search with only letters that appear in the given list
    /// of words. If the list of words is empty, an error is returned from [`WordSearch::new`].
    ///
    /// [`WordSearch::new`]: struct.WordSearch.html#method.new
    pub use_only_given_letters_in_grid: bool,

    /// Whether backward-facing directions are allowed. Backward-facing directions are any direction that is read
    /// right-to-left or down-to-up.
    pub allow_backward_words: bool,
}

/// A word search object that contains a grid of characters and a list of each word and their positions within the grid.
#[derive(Debug)]
pub struct WordSearch {
    grid: Array2D<char>,
    word_spans: Vec<(String, WordSpan)>,
}

impl WordSearch {
    /// Creates and generates a new word search with the specified configuration, or returns an error if the word search can't be created.
    ///
    /// When `config.use_only_given_letters_in_grid` is true, then the spaces in the grid that are not taken up by the given words
    /// will randomly select from all unique letters contained in the given words. As such, when this is set to true and the
    /// words list is empty, an [Error] will be returned. When `config.use_only_given_letters_in_grid` is false, any letter from 'a'
    /// to 'z' will be used to fill empty space in the grid.
    ///
    /// When `config.allow_backward_words` is false, words will only appear in down, up-right, right, and down-right directions.
    /// Otherwise, any word direction is allowed, including left-facing and up-facing directions.
    pub fn new<'a>(config: &WordSearchConfig<'a>) -> Result<Self, Error<'a>> {
        // check that the grid is big enough to hold all words
        if let Some(longest_word_length) = config.words.iter().map(|word| word.len()).max() {
            if longest_word_length > config.num_rows || longest_word_length > config.num_columns {
                return Err(Error::DimensionsTooSmall(
                    config.num_rows,
                    config.num_columns,
                    config.words,
                ));
            }
        }

        let mut grid = if config.use_only_given_letters_in_grid {
            Self::create_grid_from_words(config.num_rows, config.num_columns, config.words)?
        } else {
            Self::create_grid(config.num_rows, config.num_columns)
        };

        let spans = Self::generate_spans(
            grid.num_rows(),
            grid.num_columns(),
            config.words,
            config.allow_backward_words,
        );

        assert_eq!(
            spans.len(),
            config.words.len(),
            "There should be one word span for every word, thus their lengths must be equal. Number of spans is {} while number of words is {}",
            spans.len(),
            config.words.len(),
        );

        let word_spans: Vec<_> = config.words.iter().cloned().zip(spans).collect();

        Self::place_words(&mut grid, &word_spans);

        Ok(Self { grid, word_spans })
    }

    fn create_grid_with_letters(
        num_rows: usize,
        num_columns: usize,
        letters: &[char],
    ) -> Array2D<char> {
        let mut rng = rand::thread_rng();

        Array2D::filled_by_row_major(
            || letters[rng.gen_range(0..letters.len())],
            num_rows,
            num_columns,
        )
    }

    fn create_grid(num_rows: usize, num_columns: usize) -> Array2D<char> {
        let letters: Vec<char> = ('a'..='z').collect();
        Self::create_grid_with_letters(num_rows, num_columns, &letters)
    }

    fn create_grid_from_words<'a>(
        num_rows: usize,
        num_columns: usize,
        words: &[String],
    ) -> Result<Array2D<char>, Error<'a>> {
        if words.is_empty() {
            // we can't create the grid using letters from the given words if there are no words
            return Err(Error::NoGivenLettersToUseInGrid);
        }

        let mut letters = HashSet::new();

        for word in words {
            for ch in word.chars() {
                letters.insert(ch);
            }
        }

        let letters: Vec<char> = letters.into_iter().collect();

        Ok(Self::create_grid_with_letters(
            num_rows,
            num_columns,
            &letters,
        ))
    }

    fn generate_spans(
        num_rows: usize,
        num_columns: usize,
        words: &[String],
        allow_backward_words: bool,
    ) -> Vec<WordSpan> {
        let mut rng = rand::thread_rng();

        let mut spans: Vec<WordSpan> = Vec::with_capacity(words.len());

        let mut i = 0;
        while spans.len() < words.len() {
            let word = &words[i];

            let pos = (rng.gen_range(0..num_rows), rng.gen_range(0..num_columns));
            let len = word.len();
            let dir = if allow_backward_words {
                WordDirection::random()
            } else {
                WordDirection::random_forward()
            };

            let span = WordSpan::new(pos, len, dir);

            if span.in_bounds(num_rows, num_columns) && spans.iter().all(|s| s.overlaps(&span)) {
                // The span is valid in the grid, and it doesn't conflict with any other span, so we can add it to the list
                spans.push(span);

                // Advance to the next word
                i += 1;
            }
        }

        spans
    }

    fn place_words(grid: &mut Array2D<char>, word_spans: &[(String, WordSpan)]) {
        for (word, span) in word_spans {
            for (ch, coord) in word.chars().zip(span.indices()) {
                grid[coord] = ch;
            }
        }
    }

    /// The number of rows in the word search grid.
    pub fn num_rows(&self) -> usize {
        self.grid.num_rows()
    }

    /// The number of columns in the word search grid.
    pub fn num_columns(&self) -> usize {
        self.grid.num_columns()
    }

    /// Provides a reference to the inner word search grid.
    pub fn grid(&self) -> &Array2D<char> {
        &self.grid
    }

    /// Gets the character at the specified coordinate, returning [`Option::None`] if the coordinates are out of bounds.
    pub fn get(&self, row: usize, column: usize) -> Option<char> {
        self.grid.get(row, column).copied()
    }

    /// A list containing tuples, where the first element is a word that appears in the list, and the second element contains
    /// information about the word's location within the list.
    pub fn word_spans(&self) -> &[(String, WordSpan)] {
        &self.word_spans
    }
}

impl Index<(usize, usize)> for WordSearch {
    type Output = char;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.grid[index]
    }
}

impl Display for WordSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut words_iter = self.word_spans.iter().map(|(word, _)| word);

        for row in self.grid.rows_iter() {
            for &ch in row {
                f.write_fmt(format_args!("{} ", ch))?;
            }

            f.write_fmt(format_args!(
                "| {} \n",
                words_iter.next().unwrap_or(&String::from(""))
            ))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, WordSearch, WordSearchConfig};

    #[test]
    fn generate_word_search() {
        let words = [
            String::from("lazy"),
            String::from("panic"),
            String::from("search"),
        ];

        let word_search = WordSearch::new(&WordSearchConfig {
            num_rows: 10,
            num_columns: 10,
            words: &words,
            use_only_given_letters_in_grid: false,
            allow_backward_words: true,
        });

        assert!(word_search.is_ok())
    }

    #[test]
    fn empty_word_search() {
        let word_search = WordSearch::new(&WordSearchConfig {
            num_rows: 10,
            num_columns: 10,
            words: &[],
            use_only_given_letters_in_grid: false,
            allow_backward_words: true,
        })
        .unwrap();

        assert!(word_search.word_spans().is_empty())
    }

    #[test]
    fn grid_too_small() {
        let words = [
            String::from("magnificent"),
            String::from("shishkebab"),
            String::from("thrilling"),
        ];

        let word_search = WordSearch::new(&WordSearchConfig {
            num_rows: 5,
            num_columns: 5,
            words: &words,
            use_only_given_letters_in_grid: false,
            allow_backward_words: true,
        });

        assert!(matches!(
            word_search,
            Err(Error::DimensionsTooSmall(_, _, _))
        ))
    }

    #[test]
    fn no_given_letters_to_use_in_grid() {
        let word_search = WordSearch::new(&WordSearchConfig {
            num_rows: 10,
            num_columns: 10,
            words: &[],
            use_only_given_letters_in_grid: true,
            allow_backward_words: true,
        });

        assert!(matches!(word_search, Err(Error::NoGivenLettersToUseInGrid)))
    }
}
