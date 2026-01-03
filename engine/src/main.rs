use engine::{set_global_verbosity, ELoggingVerbosity, game::ProgramState, game::StepCommand, sim};
use engine::vlog;
use std::collections::HashMap;

fn main()
{
    set_global_verbosity(ELoggingVerbosity::Normal);

    let mut program_state = ProgramState::new();

    println!("TCG Simulator");
    println!("Commands:");
    println!("  s  -> step one phase");
    println!("  t  -> step one whole turn");
    println!("  g  -> run the current game to completion");
    println!("  d  -> run the simulation to completion for the current deck");
    println!("  r  -> run the whole simulation to completion (all decks)");
    println!("  q  -> quit");
    println!();

    let mut current_lands = 29;
    let mut current_nonlands = 31;
    let change_size = 1;

    program_state.step_mode = sim::parse_command(&read_line().trim());

    // Hill-climbing algorithm: track results and find consensus among 3+ runs
    let mut result_history: HashMap<(u32, u32), Vec<f64>> = HashMap::new();
    let mut iteration = 1;

    loop
    {
        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        println!("\n=== Iteration {} ===", iteration);
        println!("Testing land/nonland ratios centered around {} lands, {} nonlands", current_lands, current_nonlands);

        // Test three configurations: current, +1 lands, -1 lands
        let result0 = sim::try_scenario(current_lands, current_nonlands, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }

        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        let result1 = sim::try_scenario(current_lands + change_size, current_nonlands - change_size, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }

        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        let result2 = sim::try_scenario(current_lands - change_size, current_nonlands + change_size, &mut program_state);
        if program_state.step_mode == StepCommand::RunDeck
        {
            program_state.step_mode = sim::parse_command(&read_line().trim());
        }

        if program_state.step_mode == StepCommand::Quit
        {
            break;
        }

        // Track results
        result_history.entry((current_lands, current_nonlands)).or_insert_with(Vec::new).push(result0);
        result_history.entry((current_lands + change_size, current_nonlands - change_size)).or_insert_with(Vec::new).push(result1);
        result_history.entry((current_lands - change_size, current_nonlands + change_size)).or_insert_with(Vec::new).push(result2);

        // Determine which configuration was best
        let smallest_turns_to_death = result0.min(result1).min(result2);

        let (best_config_name, best_lands, best_nonlands) = if result0 == smallest_turns_to_death
        {
            ("Current ratio (no change)", current_lands, current_nonlands)
        }
        else if result1 == smallest_turns_to_death
        {
            ("More lands", current_lands + change_size, current_nonlands - change_size)
        }
        else
        {
            ("More nonlands", current_lands - change_size, current_nonlands + change_size)
        };

        println!("\nIteration {} Results:", iteration);
        println!("  Current:     {} lands, {} nonlands -> {} avg turns", current_lands, current_nonlands, result0);
        println!("  More lands:  {} lands, {} nonlands -> {} avg turns", current_lands + change_size, current_nonlands - change_size, result1);
        println!("  More nonlands: {} lands, {} nonlands -> {} avg turns", current_lands - change_size, current_nonlands + change_size, result2);
        println!("\nBest configuration: {} ({} lands, {} nonlands) -> {} avg turns",
                 best_config_name, best_lands, best_nonlands, smallest_turns_to_death);

        // Find configurations with 3+ runs
        let mut candidates: Vec<(u32, u32, f64)> = Vec::new();
        for ((lands, nonlands), results) in &result_history
        {
            if results.len() >= 3
            {
                let avg: f64 = results.iter().sum::<f64>() / results.len() as f64;
                candidates.push((*lands, *nonlands, avg));
            }
        }

        if candidates.is_empty()
        {
            // Not enough data yet, just move to the best from this iteration
            current_lands = best_lands;
            current_nonlands = best_nonlands;
            println!("Not enough data yet. Moving to best found: {} lands, {} nonlands", current_lands, current_nonlands);
        }
        else
        {
            // Find the minimum average
            let min_avg = candidates.iter().map(|(_, _, avg)| avg).fold(f64::INFINITY, |a, b| a.min(*b));
            
            // Get all candidates that match the minimum
            let tied_candidates: Vec<_> = candidates.iter()
                .filter(|(_, _, avg)| (avg - min_avg).abs() < 0.01)
                .collect();

            if tied_candidates.len() == 1
            {
                // Clear winner
                let (best_l, best_nl, best_avg) = tied_candidates[0];
                println!("\n=== Optimization Complete ===");
                vlog!(ELoggingVerbosity::Normal, "Final suggestion: {} lands, {} nonlands is optimal", best_l, best_nl);
                vlog!(ELoggingVerbosity::Normal, "Average turns to death: {:.2}", best_avg);
                break;
            }
            else
            {
                // Tiebreaker needed
                println!("\nTiebreaker needed! Testing {} tied configurations:", tied_candidates.len());
                for (lands, nonlands, avg) in &tied_candidates
                {
                    println!("  {} lands, {} nonlands -> {:.2} avg turns", lands, nonlands, avg);
                }

                let mut tiebreaker_results: Vec<(u32, u32, f64)> = Vec::new();
                for (lands, nonlands, _) in &tied_candidates
                {
                    let tiebreaker_result = sim::try_scenario(*lands, *nonlands, &mut program_state);
                    if program_state.step_mode == StepCommand::RunDeck
                    {
                        program_state.step_mode = sim::parse_command(&read_line().trim());
                    }
                    if program_state.step_mode == StepCommand::Quit
                    {
                        break;
                    }
                    tiebreaker_results.push((*lands, *nonlands, tiebreaker_result));
                }

                if program_state.step_mode == StepCommand::Quit
                {
                    break;
                }

                let tiebreaker_winner = tiebreaker_results.iter()
                    .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                    .unwrap();

                println!("\nTiebreaker winner: {} lands, {} nonlands -> {:.2} avg turns", 
                         tiebreaker_winner.0, tiebreaker_winner.1, tiebreaker_winner.2);
                println!("\n=== Optimization Complete ===");
                vlog!(ELoggingVerbosity::Normal, "Final suggestion: {} lands, {} nonlands is optimal", 
                      tiebreaker_winner.0, tiebreaker_winner.1);
                vlog!(ELoggingVerbosity::Normal, "Average turns to death: {:.2}", tiebreaker_winner.2);
                break;
            }
        }

        iteration += 1;
    }
}

fn read_line() -> String
{
    use std::io::{self, Write};
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}
