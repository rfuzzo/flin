#![warn(clippy::all, rust_2018_idioms)]

mod app;

pub use app::TemplateApp;
use egui_notify::Toasts;

use std::fmt::Display;

use log::{debug, info, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

// enums

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum EPlayer {
    PC,
    NPC,
}
impl Display for EPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EPlayer::PC => write!(f, "PC"),
            EPlayer::NPC => write!(f, "NPC"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Deserialize, Serialize)]
pub enum EValue {
    Unter = 2,
    Ober = 3,
    King = 4,
    X = 10,
    Ace = 11,
}

impl Display for EValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EValue::Unter => write!(f, "Unter"),
            EValue::Ober => write!(f, "Ober"),
            EValue::King => write!(f, "King"),
            EValue::X => write!(f, "X"),
            EValue::Ace => write!(f, "Ace"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum ESuit {
    Hearts,
    Bells,
    Acorns,
    Leaves,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum EGameState {
    None,
    PlayerTurn,
    NpcTurn,
    Evaluate,
}

impl Display for ESuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ESuit::Hearts => write!(f, "Hearts"),
            ESuit::Bells => write!(f, "Bells"),
            ESuit::Acorns => write!(f, "Acorns"),
            ESuit::Leaves => write!(f, "Leaves"),
        }
    }
}

// structs

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Card {
    pub suit: ESuit,
    pub value: EValue,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.suit, self.value)
    }
}

impl Card {
    pub fn new(suit: ESuit, value: EValue) -> Self {
        Self { suit, value }
    }
}

#[derive(Debug)]
pub struct Game {
    pub trump_suit: Option<ESuit>,
    pub trump_card: Option<Card>,
    pub talon: Vec<Card>,
    pub trick: (Option<Card>, Option<Card>),
    pub player_stack: Vec<Card>,
    pub player_hand: Vec<Card>,
    pub npc_stack: Vec<Card>,
    pub npc_hand: Vec<Card>,
    // meta
    pub forehand: Option<EPlayer>,
    pub winner: Option<EPlayer>,
    state: Option<EGameState>,
    last_turn_time: f64,
}

impl Game {
    pub fn new() -> Self {
        Self {
            trump_card: None,
            trump_suit: None,
            talon: get_deck_shuffled(),
            trick: (None, None),
            player_stack: vec![],
            player_hand: vec![],
            npc_stack: vec![],
            npc_hand: vec![],
            forehand: None,
            winner: None,
            state: None,
            last_turn_time: -1.0,
        }
    }

    /// Starts this [`Game`].
    pub fn play(&mut self, toasts: &mut Toasts, time: f64) {
        debug!("A new game has started.");
        toasts.info("A new game has started.");

        // determine who is dealer
        let mut dealer: EPlayer = EPlayer::PC;
        self.set_state(EGameState::NpcTurn, time);
        if rand::random() {
            // generates a boolean
            dealer = EPlayer::NPC;
            self.set_state(EGameState::PlayerTurn, time);
        }
        let first_player = get_opponent(dealer);
        debug!("The dealer is: {}.", dealer);
        toasts.info(format!("The dealer is: {}.", dealer));

        // deal cards
        self.deal_card(first_player);
        self.deal_card(first_player);
        self.deal_card(first_player);
        self.deal_card(dealer);
        self.deal_card(dealer);
        self.deal_card(dealer);

        self.trump_card = self.talon.pop();
        if let Some(c) = &self.trump_card {
            self.trump_suit = Some(c.suit.clone());

            debug!("Trump card is: {}.", c);
            //toasts.info(format!("Trump card is: {}.", c));
        }

        self.deal_card(first_player);
        self.deal_card(first_player);
        self.deal_card(dealer);
        self.deal_card(dealer);

        // start first turn
        if first_player == EPlayer::NPC {
            self.do_turn(toasts, time);
        }
    }

    /// Deals a card from the talon, or the trump card if the talon is empty
    ///
    /// # Panics
    ///
    /// Panics if trump card is None
    fn deal_card(&mut self, to: EPlayer) {
        let card = if self.talon.is_empty() {
            self.trump_card.take().unwrap()
        } else {
            self.talon.pop().unwrap()
        };

        match to {
            EPlayer::PC => {
                self.player_hand.push(card);
            }
            EPlayer::NPC => {
                self.npc_hand.push(card);
            }
        }
    }

    /// A turn in the game. consumes the current game state
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn do_turn(&mut self, toasts: &mut Toasts, time: f64) {
        if let Some(state) = self.state.take() {
            match state {
                EGameState::None => {}
                EGameState::PlayerTurn => {}
                EGameState::NpcTurn => {
                    let card = self.ai_choose_card();
                    self.play_card(card, EPlayer::NPC, time);
                }
                EGameState::Evaluate => self.evaluate(toasts, time),
            }
        }
    }

    /// Plays a card and evaluates the trick
    ///
    /// # Panics
    ///
    /// Panics if not forehand and no card in trick
    pub fn play_card(&mut self, card: Card, player: EPlayer, time: f64) {
        let is_forehand = self.trick.0.is_none();

        if is_forehand {
            self.forehand = Some(player);
            self.trick.0 = Some(card);

            // end turn and go to other player
            match player {
                EPlayer::PC => self.set_state(EGameState::NpcTurn, time),
                EPlayer::NPC => self.set_state(EGameState::PlayerTurn, time),
            }
        } else {
            self.trick.1 = Some(card);

            // end turn and go to evaluate
            self.set_state(EGameState::Evaluate, time);
        }
    }

    fn evaluate(&mut self, toasts: &mut Toasts, time: f64) {
        // check if backhand wins
        let backhand_wins = wins(
            self.trick.1.as_ref().unwrap(),
            self.trick.0.as_ref().unwrap(),
            self.trump_suit.clone().unwrap(),
        );

        let forehand = self.forehand.expect("There should always be a forehand, since this should only be called after a card was played");
        let winner = if backhand_wins {
            get_opponent(forehand)
        } else {
            forehand
        };

        info!("{} won this trick", winner);
        toasts.info(format!("{} won this trick", winner));

        self.give_trick_to(winner, toasts);

        if self.can_draw_card() {
            self.deal_card(winner);
            self.deal_card(get_opponent(winner));
        }

        if self.end_game(toasts) {
            return;
        }

        // winner can play again
        match winner {
            EPlayer::PC => self.set_state(EGameState::PlayerTurn, time),
            EPlayer::NPC => self.set_state(EGameState::NpcTurn, time),
        }
    }

    /// .
    fn give_trick_to(&mut self, player: EPlayer, _toasts: &mut Toasts) {
        if let Some(t1) = self.trick.0.take() {
            if let Some(t2) = self.trick.1.take() {
                match player {
                    EPlayer::PC => {
                        self.player_stack.push(t1);
                        self.player_stack.push(t2);
                    }
                    EPlayer::NPC => {
                        self.npc_stack.push(t1);
                        self.npc_stack.push(t2);
                    }
                }

                debug!("{} has {} points", player, self.get_points(player));
                //toasts.info(format!("{} has {} points", player, self.get_points(player)));
            }
        }
    }

    /// .
    fn can_draw_card(&self) -> bool {
        self.trump_card.is_some()
    }

    /// .
    // fn must_follow_suit(&self) -> bool {
    //     self.trump_card.is_none()
    // }

    /// Let the AI player choose a card
    ///
    /// # Panics
    ///
    /// Panics if .
    fn ai_choose_card(&mut self) -> Card {
        // todo ai strategy
        self.npc_hand.pop().unwrap()
    }

    /// Checks if the game should end
    fn end_game(&mut self, toasts: &mut Toasts) -> bool {
        if self.player_hand.is_empty() && self.npc_hand.is_empty() {
            info!("The game ended.");
            toasts.info("The game ended.");

            // count cards in stacks
            let player_count = self.get_points(EPlayer::PC);
            let npc_count = self.get_points(EPlayer::NPC);

            // determine winner
            if player_count > 66 {
                self.winner = Some(EPlayer::PC);
            } else if npc_count > 66 {
                self.winner = Some(EPlayer::NPC);
            }

            return true;
        }
        false
    }

    pub fn get_points(&self, player: EPlayer) -> usize {
        match player {
            EPlayer::PC => self.player_stack.iter().map(|c| c.value as usize).sum(),
            EPlayer::NPC => self.npc_stack.iter().map(|c| c.value as usize).sum(),
        }
    }

    pub fn set_state(&mut self, state: EGameState, time: f64) {
        self.state = Some(state);
        self.last_turn_time = time;
    }
}

fn wins(card: &Card, played_card: &Card, trump: ESuit) -> bool {
    if card.suit == played_card.suit {
        card.value > played_card.value
    } else {
        card.suit == trump
    }
}

// helper methods

/// .
pub fn get_deck_shuffled() -> Vec<Card> {
    let mut rng = thread_rng();
    let mut r = get_deck();
    r.shuffle(&mut rng);
    r
}

/// .
pub fn get_deck() -> Vec<Card> {
    let r = vec![
        Card::new(ESuit::Hearts, EValue::Unter),
        Card::new(ESuit::Hearts, EValue::Ober),
        Card::new(ESuit::Hearts, EValue::King),
        Card::new(ESuit::Hearts, EValue::X),
        Card::new(ESuit::Hearts, EValue::Ace),
        Card::new(ESuit::Bells, EValue::Unter),
        Card::new(ESuit::Bells, EValue::Ober),
        Card::new(ESuit::Bells, EValue::King),
        Card::new(ESuit::Bells, EValue::X),
        Card::new(ESuit::Bells, EValue::Ace),
        Card::new(ESuit::Acorns, EValue::Unter),
        Card::new(ESuit::Acorns, EValue::Ober),
        Card::new(ESuit::Acorns, EValue::King),
        Card::new(ESuit::Acorns, EValue::X),
        Card::new(ESuit::Acorns, EValue::Ace),
        Card::new(ESuit::Leaves, EValue::Unter),
        Card::new(ESuit::Leaves, EValue::Ober),
        Card::new(ESuit::Leaves, EValue::King),
        Card::new(ESuit::Leaves, EValue::X),
        Card::new(ESuit::Leaves, EValue::Ace),
    ];
    r
}

fn get_opponent(player: EPlayer) -> EPlayer {
    match player {
        EPlayer::PC => EPlayer::NPC,
        EPlayer::NPC => EPlayer::PC,
    }
}
