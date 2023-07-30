use std::fmt::Display;

use inquire::{error::InquireError, Select};
use rand::seq::SliceRandom;
use rand::thread_rng;

// enums

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug)]
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
}

impl Game {
    pub fn new() -> Self {
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
        }
    }

    /// Starts this [`Game`].
    pub fn play(&mut self) {
        println!("A new game has started.");

        // determine who is dealer
        let mut dealer: EPlayer = EPlayer::PC;
        if rand::random() {
            // generates a boolean
            dealer = EPlayer::NPC;
        }
        let first_player = get_opponent(dealer);
        println!("The dealer is: {}.", dealer);

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

            println!("Trump card is: {}.", c);
        }

        self.deal_card(first_player);
        self.deal_card(first_player);
        self.deal_card(dealer);
        self.deal_card(dealer);

        // start first turn
        self.do_turn(first_player, true);
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
        println!("= It's {}'s turn.", player);

        let card = match player {
            EPlayer::PC => {
                // wait for input
                self.player_choose_card()
            }
            EPlayer::NPC => {
                // pick a card
                self.ai_choose_card()
            }
        };

        // play card
        println!("{} was played by {}", card, player);
        let wins_or_none = self.play_card(card, is_forehand);

        if is_forehand {
            // end turn and go to other player
            self.do_turn(get_opponent(player), false);
        } else {
            // evaluate trick
            if let Some(wins) = wins_or_none {
                println!("{} won this trick: {}", player, wins);

                if wins {
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
    }

    /// Plays a card and evaluates the trick
    ///
    /// # Panics
    ///
    /// Panics if not forehand and no card in trick
    fn play_card(&mut self, card: Card, is_forehand: bool) -> Option<bool> {
        if is_forehand {
            self.trick.0 = Some(card);
            None
        } else {
            // evaluate trick
            self.trick.1 = Some(card);
            let wins_trick = wins(
                self.trick.1.as_ref().unwrap(),
                self.trick.0.as_ref().unwrap(),
                self.trump_suit.clone().unwrap(),
            );
            Some(wins_trick)
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

                println!("{} has {} points", player, self.get_points(player));
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

        let ans: Result<String, InquireError> = Select::new("Choose a card", options).prompt();

        match ans {
            Ok(choice) => {
                let index = self
                    .player_hand
                    .iter()
                    .position(|p| p.to_string() == choice)
                    .unwrap();
                let card = self.player_hand.swap_remove(index);

                // check follow suit rules
                if self.must_follow_suit() {
                    if let Some(trick) = &self.trick.0 {
                        // must follow trick suit (farbzwang)
                        if card.suit != trick.suit
                            && self.player_hand.iter().any(|c| c.suit == trick.suit)
                        {
                            println!("You violated the law!");
                        }
                        // must win (stichzwang)
                        // todo
                    }
                }

                card
            }
            Err(_) => panic!("There was an error, please try again."),
        }
    }

    /// Checks if the game should end
    fn end_game(&mut self) -> bool {
        if self.player_hand.is_empty() && self.npc_hand.is_empty() {
            println!("The game ended.");

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

    fn get_points(&self, player: EPlayer) -> usize {
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

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

// helper methods

fn get_deck() -> Vec<Card> {
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
