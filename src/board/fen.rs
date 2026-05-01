use std::fmt;

use super::{Board, CastlingRights, Color, Piece, Square};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FenError {
    WrongFieldCount,
    InvalidPiecePlacement,
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidHalfmoveClock,
    InvalidFullmoveNumber,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FenError {}

pub fn parse_fen(fen: &str) -> Result<Board, FenError> {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() != 6 {
        return Err(FenError::WrongFieldCount);
    }

    let mut board = Board::empty();
    parse_piece_placement(fields[0], &mut board)?;

    board.side_to_move = match fields[1] {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err(FenError::InvalidSideToMove),
    };

    board.castling_rights = parse_castling_rights(fields[2])?;
    board.en_passant = match fields[3] {
        "-" => None,
        square => Some(Square::from_algebraic(square).ok_or(FenError::InvalidEnPassant)?),
    };
    board.halfmove_clock = fields[4]
        .parse()
        .map_err(|_| FenError::InvalidHalfmoveClock)?;
    board.fullmove_number = fields[5]
        .parse()
        .map_err(|_| FenError::InvalidFullmoveNumber)?;

    if board.fullmove_number == 0 {
        return Err(FenError::InvalidFullmoveNumber);
    }

    board.zobrist_key = crate::search::zobrist().hash_board(&board);

    Ok(board)
}

fn parse_piece_placement(placement: &str, board: &mut Board) -> Result<(), FenError> {
    let ranks: Vec<&str> = placement.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::InvalidPiecePlacement);
    }

    for (fen_rank, rank_text) in ranks.iter().enumerate() {
        let rank = 7_u8
            .checked_sub(fen_rank as u8)
            .ok_or(FenError::InvalidPiecePlacement)?;
        let mut file = 0_u8;

        for ch in rank_text.chars() {
            if let Some(empty) = ch.to_digit(10) {
                if empty == 0 || empty > 8 {
                    return Err(FenError::InvalidPiecePlacement);
                }
                file = file
                    .checked_add(empty as u8)
                    .ok_or(FenError::InvalidPiecePlacement)?;
            } else if let Some(piece) = Piece::from_fen_char(ch) {
                let square =
                    Square::from_file_rank(file, rank).ok_or(FenError::InvalidPiecePlacement)?;
                board.set_piece(square, Some(piece));
                file += 1;
            } else {
                return Err(FenError::InvalidPiecePlacement);
            }

            if file > 8 {
                return Err(FenError::InvalidPiecePlacement);
            }
        }

        if file != 8 {
            return Err(FenError::InvalidPiecePlacement);
        }
    }

    Ok(())
}

fn parse_castling_rights(text: &str) -> Result<CastlingRights, FenError> {
    if text == "-" {
        return Ok(CastlingRights::default());
    }

    let mut rights = CastlingRights::default();
    for ch in text.chars() {
        match ch {
            'K' if !rights.white_kingside => rights.white_kingside = true,
            'Q' if !rights.white_queenside => rights.white_queenside = true,
            'k' if !rights.black_kingside => rights.black_kingside = true,
            'q' if !rights.black_queenside => rights.black_queenside = true,
            _ => return Err(FenError::InvalidCastlingRights),
        }
    }

    Ok(rights)
}
