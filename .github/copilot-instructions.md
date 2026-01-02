# AI Agent Instructions for tcg-sim

## Project Overview
A Rust-based trading card game (TCG) simulation framework designed to run game simulations and analyze deck configurations. The engine models game turns, card zones, and combat mechanics to help optimize deck composition.

## Architecture

### Core Concepts
- **GameState**: Holds the mutable game board state (`zones` HashMap, life total, lands in play, current turn/step)
- **Card/Deck**: Static card definitions with type, cost, power/toughness; Deck holds a vector of cards
- **Zones**: HashMap-based card location tracking (Library, Hand, Battlefield, Graveyard, Exile) — see `Zone` enum
- **GameStep**: Turn flow state machine (StartTurn → Draw → Main → Combat → EndTurn → GameOver) — handles game progression

### Card model (updated)

- `Card` now uses composition: it contains `card_types: Vec<CardType>` and `stats: Option<CreatureStats>`.
- Only creatures carry `CreatureStats { power, toughness }`. Use `is_type()` / `is_creature()` to query types.
- Runtime helpers: `add_type(CardType)` and `remove_type(CardType)` (removing `Creature` clears `stats`).
- Factories: `forest()` returns a non-creature card (`stats = None`); `grizzly_bears()` returns a creature with `stats = Some(...)`.

### Key Patterns to Know

**1. Borrow Management in Zone Operations**
The code frequently uses scoped blocks `{}` to release mutable borrows before re-acquiring them:
```rust
let card_option = {
    let hand = self.zones.get_mut(&Zone::Hand).unwrap();
    if let Some(pos) = hand.iter().position(|c| c.card_type == CardType::Land) {
        Some(hand.remove(pos))
    } else {
        None
    }
};
// borrow released here, safe to borrow battlefield
if let Some(card) = card_option {
    let battlefield = self.zones.get_mut(&Zone::Battlefield).unwrap();
    battlefield.push(card);
}
```
This is intentional to work around Rust's borrow checker when moving cards between zones.

**2. Verbosity Levels via Static Atomic**
Use `ELoggingVerbosity` enum with thread-safe `GLOBAL_VERBOSITY` atomic. Always check verbosity before verbose logging:
```rust
vlog!(ELoggingVerbosity::Verbose, "Cast {}", card.name);
```
Set global level via `set_global_verbosity()` in main.

**3. Simulation Loop Control**
`ProgramState` manages interactive stepping modes (StepPhase, StepTurn, RunGame, RunDeck, RunAll, Quit). `simulate_game()` returns `(turns, StepCommand)` — the new mode allows state persistence across game runs. `try_scenario()` runs 10,000 games per deck config and updates `program_state.step_mode` for interactive flow.

## Build & Run

```bash
cd engine
cargo run              # Interactive mode: step phases, turn, or auto-run
cargo build            # Release build
cargo run --release    # Optimized binary
```

Default scenario: compares three deck compositions (29/31 lands/nonlands, ±1 land variance) and suggests the best ratio based on average turns to defeat.

## TODOs in Code
- `GameStep` should split into `ETurnPhase` and `EGamePhase` (separate turn structure from game phases)
- `GameState::new()` currently takes single `Deck`; needs refactor to support multiple players

## Testing & Iteration
- Interactive mode allows single-step debugging: use `s` (phase), `t` (turn), `g` (game)
- Run large simulations with `d` (10k games for one deck) or `r` (all three scenarios)
- Adjust `lands`, `nonlands`, and `change_size` in main() to test different deck ratios
- Verbosity can be set dynamically before test runs (e.g., `VeryVerbose` for detailed logs)

## When Adding Features
1. **New card mechanics**: Add variants to `CardType` enum and handle in `GameState::step()` relevant phases
2. **New zones**: Add to `Zone` enum; initialize in `GameState::new()`
3. **Multiplayer support**: Refactor `GameState::new()` and `simulate_game()` to accept list of players/decks; track separate life totals and zones per player
4. **Phase logic**: Respect state machine pattern — each match arm in `GameState::step()` transitions exactly once to the next `GameStep`

### Notes for AI agents

- Use the `Card` API when inspecting cards: prefer `card.is_creature()` and `card.stats` over direct field access.
- When creating a creature at runtime, set `card.add_type(CardType::Creature)` and assign `card.stats = Some(CreatureStats { power, toughness })`.
- The `describe_verbose()` code groups cards using `card.is_creature()` and `card.stats` — follow this pattern for new display logic.
