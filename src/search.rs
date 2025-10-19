use std::iter::successors;

use hashbrown::HashMap;
use rand::Rng as _;
use rayon::range; // import trait to make the `random_range` method available (Rng = Random number generator)

use crate::board::*;

pub fn select_action(board: PlayableBoard) -> Option<Action> {
    //select_action_randomly(board)
    //select_action_greedily(board)
    select_action_expectimax(board, 3)
}

pub fn select_action_randomly(board: PlayableBoard) -> Option<Action> {
    // iterate through all actions and keep the applicable ones
    let mut applicable_actions: Vec<Action> = Vec::new();
    for action in ALL_ACTIONS {
        if let Some(_succ) = board.apply(action) {
            // action is applicable
            applicable_actions.push(action);
        } else {
            // action is not aplicable, ignore
        }
    }

    // if there is no available actions, return `None` immediately
    let num_actions = applicable_actions.len();
    if num_actions == 0 {
        // no available action
        return None;
    }

    // otherwise, randomly pick an action among the applicable ones
    let randomly_selected_action_index = rand::rng().random_range(0..num_actions);
    let randomly_selected_action = applicable_actions[randomly_selected_action_index];
    Some(randomly_selected_action)
}

    /*
    simulates all possible actions
    evaluate all applicable ones (the RandableBoard type provides an evaluate method)
    return the action with the highest evaluation
    or return None if there were no applicable action
    */
pub fn select_action_greedily(board: PlayableBoard) -> Option<Action> {

        // iterate through all actions and keep the applicable ones
        let mut best_action: Option<Action> =None ;
        let mut best_score: f32 = 0.0;
        for action in ALL_ACTIONS {
            if let Some(_succ) = board.apply(action) {
                // action is applicable, we check if its better than the current best
                let current_eval= _succ.evaluate();
                if current_eval > best_score{
                    best_action = Some(action);
                    best_score = current_eval;
                }
            } else {
                // action is not aplicable, ignore
            }
        }
        return best_action;
}

//select_action_expecitmax(board, max_depth):
//  applicable_actions = { actions that are applicable in board }
//  return applicable action a that maximizes eval_randable(result(board, a))
pub fn select_action_expectimax(board: PlayableBoard, max_actions: usize) -> Option<Action> {
    let mut remaining_actions:usize = max_actions;
    let mut cache: HashMap<RandableBoard, (f32, usize)> = HashMap::new();
    let mut stats = Stats::default();
    let mut best_action: Option<Action> =None ;
    let mut best_score: f32 = 0.0;
    for action in ALL_ACTIONS {
        if let Some(_succ) = board.apply(action) {
            // action is applicable, we check if its better than the current best
            let current_eval = evaluate_randable(_succ, remaining_actions-1, &mut stats, &mut cache);
            if current_eval > best_score{
                best_action = Some(action);
                best_score = current_eval;
            }
        } else {
            // action is not aplicable, ignore
        }
    }
    return best_action;
}


// eval_randable(board, remaining_actions) =
//   if remaining_actions == 0:
//     evaluate(board)
//   else
//     Sum { p * eval_action(succ, remaining_actions) | (p, succ) in successors(board) }
// we evaluate te average board depending on the placement of the 2 or 4 tile.
fn evaluate_randable(board: RandableBoard, remaining_actions: usize, stats: &mut Stats, cache:&mut HashMap<RandableBoard, (f32, usize)>) -> f32 {
    let mut sum: f32 = 0.0;
    if cache.contains_key(&board) && cache[&board].1 == remaining_actions{
        return cache[&board].0;
    }
    else if (remaining_actions == 0){ //if there is no actions possible after this state
        return board.evaluate();
    }
    else{
        for (proba, succ) in board.successors(){
            sum = sum + proba * evaluate_playable(succ, remaining_actions, stats, cache);
            cache.insert(board, (sum, remaining_actions));
        }
    }
    return sum;
}

// eval_playable(s, d) =
// applicable_actions = { actions that are applicable in s }
// successors = { result(s, action)  |  action in applicable_actions}
// max { eval_chance(succ, d-1)  | succ in successors }
// we choose the best action
fn evaluate_playable(board: PlayableBoard, remaining_actions: usize, stats: &mut Stats, cache:&mut HashMap<RandableBoard, (f32, usize)>) -> f32 {
    // iterate through all actions and keep the applicable ones
    let mut best_action: Option<Action> =None ;
    let mut best_score: f32 = 0.0;
    for action in ALL_ACTIONS {
        if let Some(_succ) = board.apply(action) {
            // action is applicable, we check if its better than the current best
            let current_eval = evaluate_randable(_succ, remaining_actions-1, stats, cache);
                if current_eval > best_score{
                best_action = Some(action);
                best_score = current_eval;
            }
        } else {
            // action is not aplicable, ignore
        }
    }
    return best_score;
}

/// A small structure to accumulated statistics accros deeply nested calls
#[derive(Default)]
struct Stats {
    /// number of time the evaluation method is called on
    pub num_evals: usize,
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Num evals: {}", self.num_evals)?;
        Ok(())
    }
}
