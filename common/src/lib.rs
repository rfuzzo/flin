use std::fmt::Display;

use log::Level;
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
    pub is_console: bool,
    // callbacks
    pub fn_notify: Option<Notification>,
    pub fn_sync_prompt: Option<SyncPrompt>,
}

type Notification = fn(&str, Level);
type SyncPrompt = fn(Vec<String>) -> String;

impl Game {
    pub fn new(
        is_console: bool,
        fn_notify: Option<Notification>,
        fn_sync_prompt: Option<SyncPrompt>,
    ) -> Self {
        Self {
            trump_card: None,
            trump_suit: None,
            talon: get_deck(),
            trick: (None, None),
            player_stack: vec![],
            player_hand: vec![],
            npc_stack: vec![],
            npc_hand: vec![],
            forehand: None,
            winner: None,
            is_console,
            fn_notify,
            fn_sync_prompt,
        }
    }

    /// Starts this [`Game`].
    pub fn play(&mut self) {
        self.log("A new game has started.", Level::Debug);

        // determine who is dealer
        let mut dealer: EPlayer = EPlayer::PC;
        if rand::random() {
            // generates a boolean
            dealer = EPlayer::NPC;
        }
        let first_player = get_opponent(dealer);
        self.log(format!("The dealer is: {}.", dealer).as_str(), Level::Debug);

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

            self.log(format!("Trump card is: {}.", c).as_str(), Level::Debug);
        }

        self.deal_card(first_player);
        self.deal_card(first_player);
        self.deal_card(dealer);
        self.deal_card(dealer);

        // start first turn
        self.do_turn(first_player, true);
    }

    fn log(&mut self, msg: &str, level: Level) {
        if let Some(notify) = self.fn_notify {
            notify(msg, level);
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

    /// A turn in the game
    ///
    /// # Panics
    ///
    /// Panics if .
    fn do_turn(&mut self, player: EPlayer, is_forehand: bool) {
        match player {
            EPlayer::PC => {
                if self.is_console {
                    let card = self.player_choose_card();
                    self.play_card(card, player, is_forehand);
                }
            }
            EPlayer::NPC => {
                let card = self.ai_choose_card();
                self.play_card(card, player, is_forehand);
            }
        };
    }

    /// Plays a card and evaluates the trick
    ///
    /// # Panics
    ///
    /// Panics if not forehand and no card in trick
    pub fn play_card(&mut self, card: Card, player: EPlayer, is_forehand: bool) {
        self.log(
            format!("{} was played by {}", card, player).as_str(),
            Level::Debug,
        );

        if is_forehand {
            self.trick.0 = Some(card);

            // end turn and go to other player
            self.do_turn(get_opponent(player), false);
        } else {
            self.trick.1 = Some(card);

            let wins = wins(
                self.trick.1.as_ref().unwrap(),
                self.trick.0.as_ref().unwrap(),
                self.trump_suit.clone().unwrap(),
            );

            // evaluate trick
            if wins {
                self.log(format!("{} won this trick", player).as_str(), Level::Info);

                self.give_trick_to(player);
                // I draw
                if self.can_draw_card() {
                    self.deal_card(player);
                    self.deal_card(get_opponent(player));
                }

                if self.end_game() {
                    return;
                }

                // can play again
                self.do_turn(player, true);
            } else {
                self.log(
                    format!("{} won this trick", get_opponent(player)).as_str(),
                    Level::Info,
                );

                self.give_trick_to(get_opponent(player));
                // opponent draws
                if self.can_draw_card() {
                    self.deal_card(get_opponent(player));
                    self.deal_card(player);
                }

                if self.end_game() {
                    return;
                }

                // loose and opponents turn
                self.do_turn(get_opponent(player), true);
            }
        }
    }

    /// .
    fn give_trick_to(&mut self, player: EPlayer) {
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

                self.log(
                    format!("{} has {} points", player, self.get_points(player)).as_str(),
                    Level::Debug,
                );
            }
        }
    }

    /// .
    fn can_draw_card(&self) -> bool {
        self.trump_card.is_some()
    }

    /// .
    fn must_follow_suit(&self) -> bool {
        self.trump_card.is_none()
    }

    /// Let the AI player choose a card
    ///
    /// # Panics
    ///
    /// Panics if .
    fn ai_choose_card(&mut self) -> Card {
        // todo ai strategy
        self.npc_hand.pop().unwrap()
    }

    /// let the player choose a card
    ///
    /// # Panics
    ///
    /// Panics if .
    fn player_choose_card(&mut self) -> Card {
        let options = self
            .player_hand
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>();

        // info
        if let Some(trick) = &self.trick.0 {
            if let Some(trump) = &self.trump_card {
                self.log(format!("[ {} ] | {}", trump, trick).as_str(), Level::Info);
            } else if let Some(trump_suit) = &self.trump_suit {
                self.log(
                    format!("[ {} ] | {}", trump_suit, trick).as_str(),
                    Level::Info,
                );
            }
        } else if let Some(trump) = &self.trump_card {
            self.log(format!("[ {} ] | ", trump).as_str(), Level::Info);
        } else if let Some(trump_suit) = &self.trump_suit {
            self.log(format!("[ {} ] | ", trump_suit).as_str(), Level::Info);
        }

        let result = if let Some(prompt) = self.fn_sync_prompt {
            prompt(options)
        } else {
            panic!("Prompt is needed in console mode.");
        };

        let index = self
            .player_hand
            .iter()
            .position(|p| p.to_string() == result)
            .unwrap();
        let card = self.player_hand.swap_remove(index);

        // check follow suit rules
        if self.must_follow_suit() {
            if let Some(trick) = &self.trick.0 {
                // must follow trick suit (farbzwang)
                if card.suit != trick.suit && self.player_hand.iter().any(|c| c.suit == trick.suit)
                {
                    self.log("You violated the law!", Level::Error);
                }
                // must win (stichzwang)
                // todo
            }
        }

        card
    }

    /// Checks if the game should end
    fn end_game(&mut self) -> bool {
        if self.player_hand.is_empty() && self.npc_hand.is_empty() {
            self.log("The game ended.", Level::Info);

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
pub fn get_deck() -> Vec<Card> {
    let mut rng = thread_rng();

    let mut r = vec![
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

    r.shuffle(&mut rng);
    r
}

fn get_opponent(player: EPlayer) -> EPlayer {
    match player {
        EPlayer::PC => EPlayer::NPC,
        EPlayer::NPC => EPlayer::PC,
    }
}
