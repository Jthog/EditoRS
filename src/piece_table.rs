use std::usize;

pub struct PieceTable {
    original_buf: Buffer,
    add_buf: Buffer,
    pieces: Vec<Piece>,
}

struct Buffer {
    contents: String,
    line_starts: Vec<usize>,
}

#[derive(Clone, Copy)]
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

impl PieceTable {
    pub fn build(contents: String) -> PieceTable {
        let mut line_starts = Vec::new();
        for (i, char) in contents.chars().enumerate() {
            if char == 0xA as char {
                line_starts.push(i);
            }
        }
        let original_buf = Buffer {
            contents,
            line_starts,
        };
        let pieces = vec![Piece {
            source: Source::Original,
            start: 0,
            length: original_buf.contents.chars().count(),
        }];
        PieceTable {
            original_buf,
            add_buf: Buffer {
                contents: String::from(""),
                line_starts: Vec::new(),
            },
            pieces,
        }
    }

    pub fn insert(&mut self, insert_str: &str, position: usize) {
        // TODO: Check for previous insert in add_buf and insert new linestart for linebreak
        let insert_length = insert_str.chars().count();
        if insert_length == 0 {
            panic!("Cannot insert String with length 0!");
        }
        let start = self.add_buf.contents.chars().count();
        let mut line_starts = Vec::new();
        for (i, char) in insert_str.chars().enumerate() {
            if char == 0xA as char {
                line_starts.push(i + start + 1);
            }
        }
        let mut insert_index: Option<usize> = None;
        let mut text_position = 0;
        let mut offset = 0;
        for (i, piece) in self.pieces.iter().enumerate() {
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
            length: insert_length,
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

        self.add_buf.contents += insert_str;
        self.add_buf.line_starts.append(&mut line_starts);
    }

    pub fn delete(&mut self, position: usize) {
        let mut text_position = 0;
        let mut to_delete: Vec<usize> = Vec::new();
        let mut new_piece: Option<Piece> = None;
        let mut new_piece_index = 0;
        for (i, piece) in self.pieces.iter_mut().enumerate() {
            if text_position + piece.length < position {
                continue;
            }

            if text_position >= position && piece.length == 1 {
                // INFO: Delete entire piece
                to_delete.push(i);
                break;
            } else if text_position >= position && piece.length > 1 {
                // INFO: Push start of piece back
                piece.start += 1;
                break;
            } else if text_position < position && text_position + piece.length > position + 1 {
                // INFO: Split piece
                new_piece_index = i;
                new_piece = Some(Piece {
                    source: piece.source,
                    start: position + 1,
                    length: piece.start + piece.length - position - 1,
                });

                piece.length = position - piece.start;
                break;
            } else if text_position < position && text_position + piece.length <= position + 1 {
                // INFO: Adjust piece length
                piece.length = position - piece.start;
                break;
            }
            text_position += piece.length;
        }

        if let Some(piece) = new_piece {
            self.pieces.insert(new_piece_index, piece);
        }

        for index in to_delete.iter() {
            self.pieces.remove(*index);
        }
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

    pub fn get_line_length(&self, line_index: usize) -> usize {
        let mut row = 0;
        let (mut start_offset, mut end_offset) = (0, 0);
        let (mut start_piece, mut end_piece) = (0, self.pieces.iter().count() - 1);
        for (i, piece) in self.pieces.iter().enumerate() {
            if row == line_index + 1 {
                break;
            }
            let buf = match piece.source {
                Source::Original => &self.original_buf,
                Source::Added => &self.add_buf,
            };

            for line_start in buf.line_starts.iter() {
                if *line_start >= piece.start || *line_start < piece.start + piece.length {
                    row += 1;

                    if row == line_index {
                        start_offset = line_start - piece.start;
                        start_piece = i;
                    } else if row == line_index + 1 {
                        end_offset = piece.start + piece.length - line_start;
                        end_piece = i;
                        break;
                    }
                }
            }
        }
        let mut line_length = match end_piece - start_piece {
            0 => self.pieces[start_piece].length - start_offset - end_offset,
            1 => {
                self.pieces[start_piece].length - start_offset + self.pieces[start_piece].length
                    - end_offset
            }
            _ => {
                let mut pieces_length = 0;
                for i in start_piece..=end_piece {
                    pieces_length += self.pieces[i].length;
                }
                pieces_length -= start_offset + end_offset;
                pieces_length
            }
        };
        // TODO: Workaround; fix later
        if line_index > 0 {
            line_length -= 1;
        }
        return line_length;
    }
}
