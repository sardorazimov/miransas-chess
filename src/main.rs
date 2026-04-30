mod bench;
mod board;
mod evaluation;
mod movegen;
mod search;
mod uci;

use std::time::Instant;

use bench::run_bench;
use board::{Board, Square};
use movegen::{
    generate_legal_moves, generate_pseudo_legal_moves, is_in_check, is_square_attacked,
    king_square, perft, perft_legal, print_moves_for_square,
};
use search::{format_pv, search_best_move, search_best_move_with_tt, search_iterative};

pub const ENGINE_NAME: &str = "MIRANSAS-CHESS";
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENGINE_AUTHOR: &str = "Sardor Azimov";

const DEFAULT_BENCH_DEPTH: u32 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
enum CliCommand {
    Demo,
    Uci,
    Bench { depth: u32 },
    Perft { depth: u32 },
    Search { depth: u32, fen: Option<String> },
    Usage,
}

fn main() {
    match parse_cli_args(std::env::args().skip(1)) {
        Ok(CliCommand::Demo) => run_demo(),
        Ok(CliCommand::Uci) => uci::run(),
        Ok(CliCommand::Bench { depth }) => run_bench_command(depth),
        Ok(CliCommand::Perft { depth }) => run_perft_command(depth),
        Ok(CliCommand::Search { depth, fen }) => run_search_command(depth, fen),
        Ok(CliCommand::Usage) | Err(()) => print_usage(),
    }
}

fn parse_cli_args<I, S>(args: I) -> Result<CliCommand, ()>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();

    match args.first().map(String::as_str) {
        None | Some("demo") => Ok(CliCommand::Demo),
        Some("uci") if args.len() == 1 => Ok(CliCommand::Uci),
        Some("bench") => {
            let depth = match args.get(1) {
                Some(text) => parse_depth(text)?,
                None => DEFAULT_BENCH_DEPTH,
            };
            if args.len() <= 2 {
                Ok(CliCommand::Bench { depth })
            } else {
                Err(())
            }
        }
        Some("perft") if args.len() == 2 => Ok(CliCommand::Perft {
            depth: parse_depth(&args[1])?,
        }),
        Some("search") if args.len() >= 2 => {
            let depth = parse_depth(&args[1])?;
            let fen = if args.len() > 2 {
                Some(args[2..].join(" "))
            } else {
                None
            };
            Ok(CliCommand::Search { depth, fen })
        }
        _ => Ok(CliCommand::Usage),
    }
}

fn parse_depth(text: &str) -> Result<u32, ()> {
    match text.parse::<u32>().map_err(|_| ())? {
        0 => Err(()),
        depth => Ok(depth),
    }
}

fn run_demo() {
    let board = Board::startpos();
    let pseudo_moves = generate_pseudo_legal_moves(&board);
    let legal_moves = generate_legal_moves(&board);

    println!("{ENGINE_NAME} {ENGINE_VERSION}");
    println!("author: {ENGINE_AUTHOR}");
    println!("startpos: {}", board.to_fen_piece_placement());
    println!("pieces: {}", board.piece_count());
    println!("pseudo moves from startpos: {}", pseudo_moves.len());
    println!("legal moves from startpos: {}", legal_moves.len());
    for depth in 1..=4 {
        let result = if depth == 4 {
            search_best_move_with_tt(&board, depth, 4)
        } else {
            search_best_move(&board, depth)
        };
        match result.best_move {
            Some(best_move) => println!(
                "best move depth {depth}: {best_move} score {} nodes {} tt_hits {}",
                result.score, result.nodes, result.tt_hits
            ),
            None => println!(
                "best move depth {depth}: none score {} nodes {} tt_hits {}",
                result.score, result.nodes, result.tt_hits
            ),
        }
    }

    let iterative = search_iterative(&board, 4, 4);
    println!("iterative depth {}:", iterative.depth);
    match iterative.best_move {
        Some(best_move) => println!("best move: {best_move}"),
        None => println!("best move: none"),
    }
    println!("score: {}", iterative.score);
    println!("nodes: {}", iterative.nodes);
    println!("tt_hits: {}", iterative.tt_hits);
    println!("pv: {}", format_pv(&iterative.principal_variation));

    println!("pseudo perft depth 1: {}", perft(&board, 1));
    println!("legal perft depth 1: {}", perft_legal(&board, 1));
    println!("legal perft depth 2: {}", perft_legal(&board, 2));
    println!("legal perft depth 3: {}", perft_legal(&board, 3));
    println!(
        "white in check: {}",
        is_in_check(&board, board::Color::White)
    );
    if let Some(white_king) = king_square(&board, board::Color::White) {
        println!(
            "white king attacked: {}",
            is_square_attacked(&board, white_king, board::Color::Black)
        );
    }

    if std::env::var_os("MIRANSAS_DEBUG_MOVES").is_some() {
        let square = Square::from_algebraic("b1").expect("debug square is valid");
        print_moves_for_square(&board, square);
    }
}

fn run_bench_command(depth: u32) {
    let result = run_bench(depth);

    println!("bench depth {}", result.depth);
    println!("positions: {}", result.positions);
    println!("nodes: {}", result.total_nodes);
    println!("tt_hits: {}", result.total_tt_hits);
    println!("time: {} ms", result.elapsed_ms);
    println!("nps: {}", result.nps);
}

fn run_perft_command(depth: u32) {
    let board = Board::startpos();
    let start = Instant::now();
    let nodes = perft_legal(&board, depth);
    let elapsed_ms = start.elapsed().as_millis();
    let nps = nodes_per_second(nodes, elapsed_ms);

    println!("perft depth {depth}");
    println!("nodes: {nodes}");
    println!("time: {elapsed_ms} ms");
    println!("nps: {nps}");
}

fn run_search_command(depth: u32, fen: Option<String>) {
    let fen = fen.unwrap_or_else(|| Board::STARTPOS_FEN.to_string());
    let Ok(board) = Board::from_fen(&fen) else {
        print_usage();
        return;
    };
    let result = search_iterative(&board, depth, 16);

    println!("search depth {depth}");
    println!("fen: {fen}");
    match result.best_move {
        Some(best_move) => println!("bestmove: {best_move}"),
        None => println!("bestmove: none"),
    }
    println!("score: {}", result.score);
    println!("nodes: {}", result.nodes);
    println!("tt_hits: {}", result.tt_hits);
    println!("pv: {}", format_pv(&result.principal_variation));
}

fn nodes_per_second(nodes: u64, elapsed_ms: u128) -> u64 {
    if elapsed_ms == 0 {
        nodes
    } else {
        ((nodes as u128 * 1000) / elapsed_ms) as u64
    }
}

fn print_usage() {
    println!(
        "{ENGINE_NAME} <command>\n\nCommands:\n  demo\n  uci\n  bench [depth]\n  perft <depth>\n  search <depth> [fen]"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_depth_one_startpos_is_twenty() {
        assert_eq!(perft_legal(&Board::startpos(), 1), 20);
    }

    #[test]
    fn cli_parser_defaults_to_demo() {
        assert_eq!(parse_cli_args(Vec::<String>::new()), Ok(CliCommand::Demo));
    }

    #[test]
    fn cli_parser_parses_bench_default_depth() {
        assert_eq!(
            parse_cli_args(["bench"]),
            Ok(CliCommand::Bench {
                depth: DEFAULT_BENCH_DEPTH
            })
        );
    }

    #[test]
    fn cli_parser_parses_perft_depth() {
        assert_eq!(
            parse_cli_args(["perft", "4"]),
            Ok(CliCommand::Perft { depth: 4 })
        );
    }

    #[test]
    fn cli_parser_parses_search_fen() {
        assert_eq!(
            parse_cli_args(["search", "3", Board::STARTPOS_FEN]),
            Ok(CliCommand::Search {
                depth: 3,
                fen: Some(Board::STARTPOS_FEN.to_string())
            })
        );
    }

    #[test]
    fn invalid_depth_is_handled() {
        assert_eq!(parse_cli_args(["perft", "nope"]), Err(()));
        assert_eq!(parse_cli_args(["search", "0"]), Err(()));
    }
}
