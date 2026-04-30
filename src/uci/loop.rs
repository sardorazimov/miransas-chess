use std::{
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};

use crate::{
    ENGINE_AUTHOR, ENGINE_NAME, ENGINE_VERSION,
    board::Board,
    search::{IterativeSearchResult, format_pv, search_iterative},
    uci::command::{Command, GoCommand, board_from_position, parse_command},
};

const DEFAULT_TT_MB: usize = 16;
const MOVETIME_MAX_DEPTH: u32 = 64;

pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    run_with_io(stdin.lock(), &mut stdout);
}

fn run_with_io<R: BufRead, W: Write>(reader: R, writer: &mut W) {
    let mut board = Board::startpos();

    for line in reader.lines() {
        let Ok(line) = line else {
            break;
        };

        match parse_command(&line) {
            Command::Uci => {
                writeln!(writer, "id name {ENGINE_NAME} {ENGINE_VERSION}")
                    .expect("write UCI id name");
                writeln!(writer, "id author {ENGINE_AUTHOR}").expect("write UCI id author");
                writeln!(writer, "uciok").expect("write uciok");
            }
            Command::IsReady => {
                writeln!(writer, "readyok").expect("write readyok");
            }
            Command::UciNewGame => {
                board = Board::startpos();
            }
            Command::Position(position) => {
                if let Some(next_board) = board_from_position(&position) {
                    board = next_board;
                }
            }
            Command::Go(go) => {
                let result = match go {
                    GoCommand::Depth(depth) => search_and_print_depths(
                        &board,
                        depth,
                        Duration::from_millis(u64::MAX),
                        writer,
                    ),
                    GoCommand::MoveTime(ms) => search_and_print_depths(
                        &board,
                        MOVETIME_MAX_DEPTH,
                        Duration::from_millis(ms),
                        writer,
                    ),
                };

                write_bestmove(&result, writer);
            }
            Command::Stop => {}
            Command::Quit => break,
            Command::Unknown => {}
        }

        writer.flush().expect("flush UCI output");
    }
}

fn search_and_print_depths<W: Write>(
    board: &Board,
    max_depth: u32,
    max_time: Duration,
    writer: &mut W,
) -> IterativeSearchResult {
    let start = Instant::now();
    let max_depth = max_depth.max(1);
    let mut latest = search_iterative(board, 1, DEFAULT_TT_MB);
    write_info(&latest, writer);

    for depth in 2..=max_depth {
        if start.elapsed() >= max_time {
            break;
        }

        latest = search_iterative(board, depth, DEFAULT_TT_MB);
        write_info(&latest, writer);

        if start.elapsed() >= max_time {
            break;
        }
    }

    latest
}

fn write_info<W: Write>(result: &IterativeSearchResult, writer: &mut W) {
    writeln!(
        writer,
        "info depth {} score cp {} nodes {} pv {}",
        result.depth,
        result.score,
        result.nodes,
        format_pv(&result.principal_variation)
    )
    .expect("write UCI info");
}

fn write_bestmove<W: Write>(result: &IterativeSearchResult, writer: &mut W) {
    match result.best_move {
        Some(best_move) => writeln!(writer, "bestmove {best_move}").expect("write bestmove"),
        None => writeln!(writer, "bestmove 0000").expect("write empty bestmove"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uci_loop_answers_basic_commands() {
        let input = b"uci\nisready\nquit\n";
        let mut output = Vec::new();

        run_with_io(&input[..], &mut output);

        let text = String::from_utf8(output).expect("valid UTF-8 output");
        assert!(text.contains(&format!("id name {ENGINE_NAME} {ENGINE_VERSION}")));
        assert!(text.contains(&format!("id author {ENGINE_AUTHOR}")));
        assert!(text.contains("uciok"));
        assert!(text.contains("readyok"));
    }
}
