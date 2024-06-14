pub struct PieceTable {
    original_buf: String,
    add_buf: String,
    pieces: Vec<Piece>,
}

struct Piece {
    source: Source,
    start: usize,
    length: usize,
}

#[derive(Copy, Clone)]
enum Source {
    Original,
    Added,
}

impl PieceTable {
    pub fn build(original_buf: String) -> PieceTable {
        let pieces = vec![Piece {
            source: Source::Original,
            start: 0,
            length: original_buf.chars().count(),
        }];
        PieceTable {
            original_buf,
            add_buf: String::from(""),
            pieces,
        }
    }

    pub fn insert(&mut self, insert_str: &str, position: usize) {
        // TODO: Check for previous insert in add_buf
        let insert_length = insert_str.chars().count();
        if insert_length == 0 {
            panic!("Cannot insert String with length 0!");
        }
        let start = self.add_buf.chars().count();
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

        self.add_buf += insert_str;
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
        let original_chars: Vec<char> = self.original_buf.chars().collect();
        let added_chars: Vec<char> = self.add_buf.chars().collect();

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
}
