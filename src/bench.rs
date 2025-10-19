#![allow(unused)]

use std::time::{Duration, Instant};

use anyhow::Context;
use board::PlayableBoard;
use clap::Parser;
use rayon::prelude::*;

mod board;
mod eval;
mod search;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Time in seconds allowed for a single game
    #[arg(short, long, default_value = "600")]
    timeout: u64,

    /// Number of games to play
    #[arg(short, long, default_value = "8")]
    num_games: u64,
}

fn main() -> anyhow::Result<()> {
    // retrieve command line arguments
    let args: Args = Args::parse();

    // number of game to play
    let num_games = args.num_games;
    // maximum allow runtime for each game
    let timeout = Duration::from_secs(args.timeout);

    // configure the global thread pool of rayon to have as many threads as we have *physical* CPUs
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global()
        .unwrap();

    // run all games on the thread pool and collect the results
    let results: Vec<_> = (0..num_games)
        .into_par_iter()
        .map(|_i| play(timeout))
        .collect();

    // print all results
    for res in &results {
        match res {
            // This line now works correctly due to Display implementation in board.rs
            Ok((score, board)) => println!("score (#actions): {score}\n{board}\n"),
            Err(e) => println!("{e}"),
        }
    }

    // print statistic over the valid runs
    let valid_results: Vec<_> = results.iter().filter_map(|x| x.as_ref().ok()).collect();
    println!("How many time a tile was reached:");
    for tile in 3..=15 {
        let mut count = 0;
        for (_, board) in &valid_results {
            if board.has_at_least_tile(tile) {
                count += 1;
            }
        }
        println!(
            "{:>6}: {:>6.2}%",
            2u32.pow(tile as u32),
            (count as f32) / (num_games as f32) * 100.0
        );
    }
    println!("\nNumber of successful games: {}", valid_results.len());
    println!(
        "Number of game with error:  {}",
        results.len() - valid_results.len()
    );
    let average_score: f32 =
        valid_results.iter().map(|(score, _)| *score).sum::<f32>() / (valid_results.len() as f32);
    println!("Average score (#actions):   {:6.2}", average_score);

    Ok(())
}

/// Play a game with the given `timeout
fn play(timeout: Duration) -> anyhow::Result<(f32, PlayableBoard)> {
    // timestamp of when we started to play
    let start = Instant::now();

    // count of the number of move played
    let mut num_moves = 0;
    let mut board = PlayableBoard::init();

    loop {
        let Some(action) = crate::search::select_action(board) else {
            println!("End game // num moves {num_moves}");
            return Ok((num_moves as f32, board));
        };

        if start.elapsed() > timeout {
            println!("Timeout // num moves: {num_moves}");
            return Ok((num_moves as f32, board));
        }

        //println!("GOT ========================> {action:?}");
        num_moves += 1;
        let played = board
            .apply(action)
            // This 'format!' call now works because PlayableBoard implements Display
            .with_context(|| format!("Got inapplicable action {action:?} on board\n{board}"))?;
        board = played.with_random_tile();
    }
}
