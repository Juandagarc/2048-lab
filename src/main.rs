#![allow(unused)]

pub mod board;
pub mod eval;
pub mod search;

use std::{
    time::{Instant, Duration},
    io::{self, Write},
};

use board::*;
use macroquad::prelude::*; 

// Constant for the window dimension
const WINDOW_DIM: f32 = 600.0;
// Slowdown factor for the agent, to make the game visible
const AGENT_DELAY_MS: u64 = 100;

// The main function for Macroquad must be ASYNCHRONOUS
#[macroquad::main("2048 Expectimax")]
async fn main() {
    // Set the window size
    request_new_screen_size(WINDOW_DIM, WINDOW_DIM + 60.0); // +60px for the UI

    // Mode Selection Logic 
    println!("Welcome to 2048!");
    println!("Choose the game mode:");
    println!("  [A] - Agent Mode "); // Expectimax
    println!("  [P] - Human Mode "); // Keyboard

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");
    let choice = choice.trim().to_uppercase();

    let init = PlayableBoard::init();

    match choice.as_str() {
        "A" => {
            println!("\nStarting game in Agent Mode. (Popup Window)");
            // Execute the agent's asynchronous game loop
            play_agent(init).await;
        }
        "P" => {
            println!("\nStarting game in Human Mode. (Popup Window)");
            // Execute the human player's asynchronous game loop
            play_person(init).await;
        }
        _ => {
            println!("Invalid option. Closing...");
            // If the option is invalid, show the window briefly before closing
            while !is_key_pressed(KeyCode::Escape) {
                clear_background(RED);
                draw_text("Invalid option. Press ESC.", 50.0, 300.0, 50.0, BLACK);
                next_frame().await;
            }
        }
    }
}

// Function for the Agent game mode (ASYNC)
pub async fn play_agent(init: PlayableBoard) {
    let mut num_moves = 0;
    let mut cur = init;
    let mut decision_time_ms = 0.0;
    let mut game_over = false;

    // Main Macroquad loop
    loop {
        // Rendering 
        cur.draw(num_moves, decision_time_ms);
        if game_over {
            draw_text("GAME OVER!", WINDOW_DIM/2.0 - 150.0, WINDOW_DIM/2.0 + 30.0, 80.0, RED);
            next_frame().await;
            continue;
        }
        
        // Use a frame loop to implement a non-blocking PAUSE for visibility.
        // This replaces the blocking thread::sleep.
        for _ in 0..10 { // 10 frames at 60 FPS is ~166ms pause
            cur.draw(num_moves, decision_time_ms);
            next_frame().await;
        }

        // Start action selection time measurement
        let start_action_selection = Instant::now();
        let action = match search::select_action(cur) {
            Some(action) => action,
            None => {
                // Game Over: No possible moves left
                println!("GAME OVER! Num moves: {num_moves}");
                game_over = true;
                continue;
            }
        };
        // Calculate decision time
        decision_time_ms = start_action_selection.elapsed().as_secs_f64() * 1000.0;
        println!("\n[Agent | {:.2}ms] Playing action {action:?}", decision_time_ms);

        // Apply the move
        let played = cur.apply(action).expect("invalid action");
        num_moves += 1;

        // CHANCE turn: Add a random tile
        cur = played.with_random_tile();

        // Wait for the next Macroquad frame
        next_frame().await;
    }
}

// Function for the Human player game mode (ASYNC)
pub async fn play_person(init: PlayableBoard) {
    let mut num_moves = 0;
    let mut cur = init;
    let decision_time_ms = 0.0; // Time is always 0.0 in human mode
    let mut game_over = false;

    // Main Macroquad loop
    loop {
        // --- Rendering ---
        cur.draw(num_moves, decision_time_ms);
        if game_over {
            draw_text("GAME OVER!", WINDOW_DIM/2.0 - 150.0, WINDOW_DIM/2.0 + 30.0, 80.0, RED);
            next_frame().await;
            continue;
        }

        // 0. Game Over check
        let mut is_game_over = true;
        for action in ALL_ACTIONS {
            if cur.apply(action).is_some() {
                is_game_over = false;
                break;
            }
        }

        if is_game_over {
            println!("GAME OVER! Number of moves: {num_moves}");
            game_over = true;
            next_frame().await;
            continue;
        }

        // 1. Get user action (Macroquad keyboard input)
        let mut action: Option<Action> = None;
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) { action = Some(Action::Up); }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) { action = Some(Action::Down); }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) { action = Some(Action::Left); }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) { action = Some(Action::Right); }

        if let Some(act) = action {
            // 2. Check if the action is applicable (legal move)
            if cur.apply(act).is_some() {
                // Valid action: apply move and proceed to CHANCE turn
                num_moves += 1;
                println!("[Player] Playing action {act:?}");

                // Apply the move
                let played = cur.apply(act).unwrap();

                // CHANCE turn: Add a random tile
                cur = played.with_random_tile();

                // Draw the new state before waiting for the next input
                cur.draw(num_moves, decision_time_ms);
                // Wait one frame to register the change
                next_frame().await;
            } else {
                // Invalid move (no change)
            }
        }

        // Wait for the next frame
        next_frame().await;
    }
}
