mod bench;
mod board;
mod evaluation;
mod movegen;
mod search;
mod uci;

use std::time::Instant;

use bench::{BenchResult, run_bench};
use board::{Board, Square};
use movegen::{
    generate_legal_moves, generate_pseudo_legal_moves, is_in_check, is_square_attacked,
    king_square, perft, perft_legal, print_moves_for_square,
};
use search::{
    IterativeSearchResult, format_pv, search_best_move, search_best_move_with_tt, search_iterative,
};

pub const ENGINE_NAME: &str = "MIRANSAS-CHESS";
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENGINE_AUTHOR: &str = "Sardor Azimov";

const DEFAULT_BENCH_DEPTH: u32 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CliCommand {
    Demo,
    Uci,
    Bench {
        depth: u32,
        format: OutputFormat,
    },
    Perft {
        depth: u32,
        format: OutputFormat,
    },
    Search {
        depth: u32,
        fen: Option<String>,
        format: OutputFormat,
    },
    Usage,
}

fn main() {
    match parse_cli_args(std::env::args().skip(1)) {
        Ok(CliCommand::Demo) => run_demo(),
        Ok(CliCommand::Uci) => uci::run(),
        Ok(CliCommand::Bench { depth, format }) => println!("{}", bench_output(depth, format)),
        Ok(CliCommand::Perft { depth, format }) => println!("{}", perft_output(depth, format)),
        Ok(CliCommand::Search { depth, fen, format }) => match search_output(depth, fen, format) {
            Some(output) => println!("{output}"),
            None => print_usage(),
        },
        Ok(CliCommand::Usage) | Err(()) => print_usage(),
    }
}

fn parse_cli_args<I, S>(args: I) -> Result<CliCommand, ()>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args: Vec<String> = args.into_iter().map(Into::into).collect();
    let format = extract_output_format(&mut args);

    match args.first().map(String::as_str) {
        None | Some("demo") => Ok(CliCommand::Demo),
        Some("uci") if args.len() == 1 => Ok(CliCommand::Uci),
        Some("bench") => {
            let depth = match args.get(1) {
                Some(text) => parse_depth(text)?,
                None => DEFAULT_BENCH_DEPTH,
            };
            if args.len() <= 2 {
                Ok(CliCommand::Bench { depth, format })
            } else {
                Err(())
            }
        }
        Some("perft") if args.len() == 2 => Ok(CliCommand::Perft {
            depth: parse_depth(&args[1])?,
            format,
        }),
        Some("search") if args.len() >= 2 => {
            let depth = parse_depth(&args[1])?;
            let fen = if args.len() > 2 {
                Some(args[2..].join(" "))
            } else {
                None
            };
            Ok(CliCommand::Search { depth, fen, format })
        }
        _ => Ok(CliCommand::Usage),
    }
}

fn extract_output_format(args: &mut Vec<String>) -> OutputFormat {
    if let Some(index) = args.iter().position(|arg| arg == "--json") {
        args.remove(index);
        OutputFormat::Json
    } else {
        OutputFormat::Text
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

    let mut perft_board = board.clone();
    println!("pseudo perft depth 1: {}", perft(&mut perft_board, 1));
    println!("legal perft depth 1: {}", perft_legal(&mut perft_board, 1));
    println!("legal perft depth 2: {}", perft_legal(&mut perft_board, 2));
    println!("legal perft depth 3: {}", perft_legal(&mut perft_board, 3));
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

fn bench_output(depth: u32, format: OutputFormat) -> String {
    let result = run_bench(depth);

    match format {
        OutputFormat::Text => format_bench_text(&result),
        OutputFormat::Json => format_bench_json(&result),
    }
}

fn perft_output(depth: u32, format: OutputFormat) -> String {
    let mut board = Board::startpos();
    let start = Instant::now();
    let nodes = perft_legal(&mut board, depth);
    let elapsed_ms = start.elapsed().as_millis();
    let nps = nodes_per_second(nodes, elapsed_ms);

    match format {
        OutputFormat::Text => {
            format!("perft depth {depth}\nnodes: {nodes}\ntime: {elapsed_ms} ms\nnps: {nps}")
        }
        OutputFormat::Json => format_perft_json(depth, nodes, elapsed_ms, nps),
    }
}

fn search_output(depth: u32, fen: Option<String>, format: OutputFormat) -> Option<String> {
    let fen = fen.unwrap_or_else(|| Board::STARTPOS_FEN.to_string());
    let board = Board::from_fen(&fen).ok()?;
    let result = search_iterative(&board, depth, 16);

    Some(match format {
        OutputFormat::Text => format_search_text(depth, &fen, &result),
        OutputFormat::Json => format_search_json(depth, &fen, &result),
    })
}

fn format_bench_text(result: &BenchResult) -> String {
    format!(
        "bench depth {}\npositions: {}\nnodes: {}\ntt_hits: {}\ntime: {} ms\nnps: {}",
        result.depth,
        result.positions,
        result.total_nodes,
        result.total_tt_hits,
        result.elapsed_ms,
        result.nps
    )
}

fn format_search_text(depth: u32, fen: &str, result: &IterativeSearchResult) -> String {
    let best_move = result
        .best_move
        .map(|mv| mv.to_string())
        .unwrap_or_else(|| "none".to_string());

    format!(
        "search depth {depth}\nfen: {fen}\nbestmove: {best_move}\nscore: {}\nnodes: {}\ntt_hits: {}\npv: {}",
        result.score,
        result.nodes,
        result.tt_hits,
        format_pv(&result.principal_variation)
    )
}

fn format_search_json(depth: u32, fen: &str, result: &IterativeSearchResult) -> String {
    let best_move = result
        .best_move
        .map(|mv| format!("\"{mv}\""))
        .unwrap_or_else(|| "null".to_string());
    let pv = result
        .principal_variation
        .iter()
        .map(|mv| format!("\"{mv}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "{{\"type\":\"search\",\"engine\":\"{}\",\"version\":\"{}\",\"depth\":{},\"fen\":\"{}\",\"bestMove\":{},\"score\":{},\"nodes\":{},\"ttHits\":{},\"pv\":[{}]}}",
        json_escape(ENGINE_NAME),
        json_escape(ENGINE_VERSION),
        depth,
        json_escape(fen),
        best_move,
        result.score,
        result.nodes,
        result.tt_hits,
        pv
    )
}

fn format_bench_json(result: &BenchResult) -> String {
    format!(
        "{{\"type\":\"bench\",\"engine\":\"{}\",\"version\":\"{}\",\"depth\":{},\"positions\":{},\"nodes\":{},\"ttHits\":{},\"elapsedMs\":{},\"nps\":{}}}",
        json_escape(ENGINE_NAME),
        json_escape(ENGINE_VERSION),
        result.depth,
        result.positions,
        result.total_nodes,
        result.total_tt_hits,
        result.elapsed_ms,
        result.nps
    )
}

fn format_perft_json(depth: u32, nodes: u64, elapsed_ms: u128, nps: u64) -> String {
    format!(
        "{{\"type\":\"perft\",\"engine\":\"{}\",\"version\":\"{}\",\"depth\":{},\"nodes\":{},\"elapsedMs\":{},\"nps\":{}}}",
        json_escape(ENGINE_NAME),
        json_escape(ENGINE_VERSION),
        depth,
        nodes,
        elapsed_ms,
        nps
    )
}

fn json_escape(input: &str) -> String {
    let mut escaped = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn nodes_per_second(nodes: u64, elapsed_ms: u128) -> u64 {
    (nodes as u128 * 1000)
        .checked_div(elapsed_ms)
        .map(|n| n as u64)
        .unwrap_or(nodes)
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
        assert_eq!(perft_legal(&mut Board::startpos(), 1), 20);
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
                depth: DEFAULT_BENCH_DEPTH,
                format: OutputFormat::Text
            })
        );
    }

    #[test]
    fn cli_parser_parses_perft_depth() {
        assert_eq!(
            parse_cli_args(["perft", "4"]),
            Ok(CliCommand::Perft {
                depth: 4,
                format: OutputFormat::Text
            })
        );
    }

    #[test]
    fn cli_parser_parses_search_fen() {
        assert_eq!(
            parse_cli_args(["search", "3", Board::STARTPOS_FEN]),
            Ok(CliCommand::Search {
                depth: 3,
                fen: Some(Board::STARTPOS_FEN.to_string()),
                format: OutputFormat::Text
            })
        );
    }

    #[test]
    fn cli_parser_json_works_at_end() {
        assert_eq!(
            parse_cli_args(["search", "3", Board::STARTPOS_FEN, "--json"]),
            Ok(CliCommand::Search {
                depth: 3,
                fen: Some(Board::STARTPOS_FEN.to_string()),
                format: OutputFormat::Json
            })
        );
    }

    #[test]
    fn json_escape_escapes_quotes_and_backslashes() {
        assert_eq!(json_escape("a\"b\\c\n\r\t"), "a\\\"b\\\\c\\n\\r\\t");
    }

    #[test]
    fn search_json_output_starts_with_object_and_contains_best_move() {
        let output = search_output(1, None, OutputFormat::Json).expect("valid search output");

        assert!(output.starts_with('{'));
        assert!(output.contains("\"bestMove\""));
    }

    #[test]
    fn bench_json_output_contains_positions() {
        let output = bench_output(1, OutputFormat::Json);

        assert!(output.starts_with('{'));
        assert!(output.contains("\"positions\":"));
    }

    #[test]
    fn perft_json_depth_one_contains_twenty_nodes() {
        let output = perft_output(1, OutputFormat::Json);

        assert!(output.starts_with('{'));
        assert!(output.contains("\"depth\":1"));
        assert!(output.contains("\"nodes\":20"));
    }

    #[test]
    fn normal_text_output_remains_text() {
        let output = perft_output(1, OutputFormat::Text);

        assert!(output.starts_with("perft depth 1"));
        assert!(output.contains("nodes: 20"));
        assert!(!output.starts_with('{'));
    }

    #[test]
    fn invalid_depth_is_handled() {
        assert_eq!(parse_cli_args(["perft", "nope"]), Err(()));
        assert_eq!(parse_cli_args(["search", "0"]), Err(()));
    }
}
