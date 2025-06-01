use core::fmt;
use std::usize;

pub struct PieceTable {
    original_buf: Buffer,
    add_buf: Buffer,
    pieces: Vec<Piece>,
<<<<<<< HEAD
=======
    pieces_history: Vec<Vec<Piece>>,
    history_pos: usize,
>>>>>>> 95204e2 (General rework and expansion to preliminary working condition.)
}

struct Buffer {
    contents: String,
}

#[derive(Clone)]
struct Piece {
    source: Source,
    start: usize,
    length: usize,
}

#[derive(Copy, Clone, PartialEq)]
enum Source {
    Original,
    Added,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Source::Original => write!(f, "Original"),
            Source::Added => write!(f, "Added"),
        }
    }
}

impl PieceTable {
<<<<<<< HEAD
    pub fn build(contents: String) -> PieceTable {
=======
    pub fn new(contents: String) -> PieceTable {
>>>>>>> 95204e2 (General rework and expansion to preliminary working condition.)
        let original_buf = Buffer { contents };
        let pieces = vec![Piece {
            source: Source::Original,
            start: 0,
            length: original_buf.contents.chars().count(),
        }];
        PieceTable {
            original_buf,
            add_buf: Buffer {
                contents: String::from(""),
            },
<<<<<<< HEAD
            pieces,
=======
            pieces: pieces.clone(),
            pieces_history: vec![pieces],
            history_pos: 0,
        }
    }

    fn store(&mut self) {
        self.history_pos = self.history_pos + 1;
        self.pieces_history.resize(self.history_pos, Vec::new());
        self.pieces_history.push(self.pieces.clone());
    }

    pub fn undo(&mut self) {
        if self.history_pos > 0 {
            self.history_pos = self.history_pos - 1;
            self.pieces = self.pieces_history[self.history_pos].clone();
        }
    }

    pub fn redo(&mut self) {
        if self.history_pos < self.pieces_history.len() - 1 {
            self.history_pos = self.history_pos + 1;
            self.pieces = self.pieces_history[self.history_pos].clone();
>>>>>>> 95204e2 (General rework and expansion to preliminary working condition.)
        }
    }

    pub fn insert(&mut self, insert_char: char, position: usize) {
        // TODO: Check for previous insert
        let start = self.add_buf.contents.chars().count();
        let mut insert_index: Option<usize> = None;
        let mut text_position = 0;
        let mut offset = 0;
        let add_buf_length = self.add_buf.contents.chars().count();
        for (i, piece) in self.pieces.iter().enumerate() {
            if piece.source == Source::Added && piece.start + piece.length == add_buf_length {
                // TODO: append if new piece at insert_index + 1
            }
            if text_position == position {
                insert_index = Some(i);
            } else if text_position < position && text_position + piece.length > position {
                insert_index = Some(i + 1);
                offset = position - text_position;
            }
            text_position += piece.length;
        }

        if position > text_position {
            panic!("String insert out of bounds!");
        }

        let new_piece = Piece {
            source: Source::Added,
            start,
            length: 1,
        };

        if offset > 0 {
            let i = insert_index.unwrap();
            let split_off_piece = {
                let split_piece = &self.pieces[i - 1];

                Piece {
                    source: split_piece.source,
                    start: split_piece.start + offset,
                    length: split_piece.length - offset,
                }
            };
            self.pieces[i - 1].length = offset;
            self.pieces.insert(i, new_piece);
            self.pieces.insert(i + 1, split_off_piece);
        } else if let Some(i) = insert_index {
            self.pieces.insert(i, new_piece);
        } else {
            self.pieces.push(new_piece);
        }

        self.add_buf.contents.push(insert_char);
<<<<<<< HEAD
=======

        self.store();
>>>>>>> 95204e2 (General rework and expansion to preliminary working condition.)
    }

    pub fn delete(&mut self, position: usize) {
        let mut cursor_pos = 0;
        for (i, piece) in self.pieces.iter_mut().enumerate() {
            if cursor_pos + piece.length - 1 < position {
                cursor_pos += piece.length;
            } else if piece.length == 1 {
                // INFO: Full: delete piece
                self.pieces.remove(i);
                break;
            } else if cursor_pos + piece.length - 1 == position {
                // INFO: End: shorten piece by one
                piece.length -= 1;
                break;
            } else if cursor_pos == position {
                // INFO: Start: shorten and shift piece by one
                piece.start += 1;
                piece.length -= 1;
                break;
            } else {
                // INFO: Middle: split piece in two
                let new_length = position - cursor_pos;
                let new_piece = Piece {
                    source: piece.source,
                    start: piece.start + new_length + 1,
                    length: piece.length - new_length - 1,
                };
                piece.length = new_length;
                self.pieces.insert(i + 1, new_piece);
                break;
            }
        }
<<<<<<< HEAD
=======
        self.store();
>>>>>>> 95204e2 (General rework and expansion to preliminary working condition.)
    }

    pub fn read(&self) -> String {
        let mut output = String::from("");
        let original_chars: Vec<char> = self.original_buf.contents.chars().collect();
        let added_chars: Vec<char> = self.add_buf.contents.chars().collect();

        for piece in self.pieces.iter() {
            let string: String = match &piece.source {
                Source::Original => original_chars[piece.start..(piece.start + piece.length)]
                    .iter()
                    .collect(),
                Source::Added => added_chars[piece.start..(piece.start + piece.length)]
                    .iter()
                    .collect(),
            };
            output += &string;
        }
        output
    }

<<<<<<< HEAD
    pub fn get_line_length(&self, line_index: usize) -> usize {
        let mut line_length = 0;
        let text = self.read();
        if let Some(line) = text.lines().nth(line_index) {
=======
    pub fn get_line_length(&self, line_index: u16) -> usize {
        let mut line_length = 0;
        let text = self.read();
        if let Some(line) = text.lines().nth(line_index.into()) {
>>>>>>> 95204e2 (General rework and expansion to preliminary working condition.)
            line_length = line.chars().count();
        }
        return line_length;
    }

    pub fn get_pieces(&self) -> String {
        let mut json = String::from("[");
        for piece in &self.pieces {
            json += &format!(
                "{{\n  \"source\": \"{}\",\n  \"start\": {},\n  \"length\": {}\n}},",
                piece.source, piece.start, piece.length
            )
            .to_string();
        }
        if self.pieces.len() > 0 {
            json.pop();
        }
        json += "]";
        return json;
    }
}
