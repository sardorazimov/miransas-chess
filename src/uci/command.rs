use crate::{
    board::Board,
    movegen::{Move, generate_legal_moves},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Uci,
    IsReady,
    UciNewGame,
    Position(PositionSpec),
    Go(GoCommand),
    Stop,
    Quit,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PositionSpec {
    Startpos { moves: Vec<String> },
    Fen { fen: String, moves: Vec<String> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoCommand {
    Depth(u32),
    MoveTime(u64),
}

pub fn parse_command(line: &str) -> Command {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let Some(command) = tokens.first() else {
        return Command::Unknown;
    };

    match *command {
        "uci" => Command::Uci,
        "isready" => Command::IsReady,
        "ucinewgame" => Command::UciNewGame,
        "stop" => Command::Stop,
        "quit" => Command::Quit,
        "position" => parse_position(&tokens[1..])
            .map(Command::Position)
            .unwrap_or(Command::Unknown),
        "go" => parse_go(&tokens[1..])
            .map(Command::Go)
            .unwrap_or(Command::Unknown),
        _ => Command::Unknown,
    }
}

pub fn parse_uci_move(board: &Board, text: &str) -> Option<Move> {
    generate_legal_moves(board)
        .into_iter()
        .find(|mv| mv.to_string() == text)
}

pub fn board_from_position(position: &PositionSpec) -> Option<Board> {
    match position {
        PositionSpec::Startpos { moves } => apply_moves(Board::startpos(), moves),
        PositionSpec::Fen { fen, moves } => Board::from_fen(fen)
            .ok()
            .and_then(|board| apply_moves(board, moves)),
    }
}

fn parse_position(tokens: &[&str]) -> Option<PositionSpec> {
    match tokens.first().copied()? {
        "startpos" => {
            let moves = parse_moves_after(&tokens[1..])?;
            Some(PositionSpec::Startpos { moves })
        }
        "fen" => {
            let mut fen_parts = Vec::new();
            let mut index = 1;

            while index < tokens.len() && tokens[index] != "moves" {
                fen_parts.push(tokens[index]);
                index += 1;
            }

            if fen_parts.len() != 6 {
                return None;
            }

            let moves = if index < tokens.len() {
                parse_moves_after(&tokens[index..])?
            } else {
                Vec::new()
            };

            Some(PositionSpec::Fen {
                fen: fen_parts.join(" "),
                moves,
            })
        }
        _ => None,
    }
}

fn parse_moves_after(tokens: &[&str]) -> Option<Vec<String>> {
    if tokens.is_empty() {
        return Some(Vec::new());
    }

    if tokens.first().copied()? != "moves" {
        return None;
    }

    Some(tokens[1..].iter().map(|text| (*text).to_string()).collect())
}

fn parse_go(tokens: &[&str]) -> Option<GoCommand> {
    let mut index = 0;
    while index + 1 < tokens.len() {
        match tokens[index] {
            "depth" => return tokens[index + 1].parse().ok().map(GoCommand::Depth),
            "movetime" => return tokens[index + 1].parse().ok().map(GoCommand::MoveTime),
            _ => index += 1,
        }
    }

    None
}

fn apply_moves(mut board: Board, moves: &[String]) -> Option<Board> {
    for text in moves {
        let mv = parse_uci_move(&board, text)?;
        board.make_move(mv);
    }

    Some(board)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::PieceKind;

    #[test]
    fn parse_uci_command() {
        assert_eq!(parse_command("uci"), Command::Uci);
    }

    #[test]
    fn parse_isready_command() {
        assert_eq!(parse_command("isready"), Command::IsReady);
    }

    #[test]
    fn parse_position_startpos() {
        assert_eq!(
            parse_command("position startpos"),
            Command::Position(PositionSpec::Startpos { moves: Vec::new() })
        );
    }

    #[test]
    fn parse_position_startpos_with_moves() {
        assert_eq!(
            parse_command("position startpos moves e2e4 e7e5"),
            Command::Position(PositionSpec::Startpos {
                moves: vec!["e2e4".to_string(), "e7e5".to_string()]
            })
        );
    }

    #[test]
    fn parse_position_fen() {
        let fen = "8/8/8/8/8/8/8/4K3 w - - 0 1";

        assert_eq!(
            parse_command(&format!("position fen {fen}")),
            Command::Position(PositionSpec::Fen {
                fen: fen.to_string(),
                moves: Vec::new()
            })
        );
    }

    #[test]
    fn parse_legal_uci_move() {
        let board = Board::startpos();

        assert_eq!(
            parse_uci_move(&board, "e2e4"),
            Some(Move::new(square("e2"), square("e4")))
        );
    }

    #[test]
    fn parse_promotion_move() {
        let board = Board::from_fen("7k/4P3/8/8/8/8/8/4K3 w - - 0 1").expect("valid FEN");

        assert_eq!(
            parse_uci_move(&board, "e7e8q"),
            Some(Move::promotion(
                square("e7"),
                square("e8"),
                PieceKind::Queen
            ))
        );
    }

    #[test]
    fn illegal_move_returns_none() {
        let board = Board::startpos();

        assert_eq!(parse_uci_move(&board, "e2e5"), None);
    }

    fn square(algebraic: &str) -> crate::board::Square {
        crate::board::Square::from_algebraic(algebraic).expect("test square is valid")
    }
}
