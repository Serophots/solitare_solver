use crate::solitare::card::Card;
use crate::solitare::solitare::{AceStack, Game};

#[derive(Debug)]
pub struct GameState<'a> { //Describes all the possible moves of the game at its current stage - not necessarily that they're good ones
    pub table_flip_moves: Vec<GameMove>, //Unflipping downward facing cards when nothing is ontop of them
    pub table_ace_moves: Vec<GameMove>,
    pub draw_ace_moves: Vec<GameMove>,
    pub table_king_moves: Vec<GameMove>,
    pub ace_stack_moves: Vec<GameMove>,
    pub table_moves: Vec<GameMove>,
    pub deck_moves: Vec<GameMove>,

    pub aces: &'a[AceStack<'a>; 4],
    pub queuing_kings: i8,
}

impl<'a> GameState<'a> {
    pub fn get_final_state(&self) -> GameFinalState {

        if self.table_flip_moves.len() == 0
            && self.table_ace_moves.len() == 0
            && self.draw_ace_moves.len() == 0
            && self.table_king_moves.len() == 0
            && self.ace_stack_moves.len() == 0
            && self.table_moves.len() == 0
            && self.deck_moves.len() == 0
        {

            if self.aces[0].is_full()
                && self.aces[1].is_full()
                && self.aces[2].is_full()
                && self.aces[3].is_full()
            {
                GameFinalState::WON
            } else {
                GameFinalState::LOST
            }

        } else {

            GameFinalState::UNFINISHED

        }
    }

    //Function exists only for debugging
    pub fn get_all_moves_youch(&self) -> Vec<&GameMove> {
        let mut moves = Vec::new();

        moves.extend(&self.table_flip_moves);
        moves.extend(&self.table_ace_moves);
        moves.extend(&self.draw_ace_moves);
        moves.extend(&self.table_king_moves);
        moves.extend(&self.ace_stack_moves);
        moves.extend(&self.table_moves);
        moves.extend(&self.deck_moves);

        moves
    }
}


#[derive(Eq, PartialEq)]
pub enum GameFinalState {
    WON,
    LOST,
    UNFINISHED
}

#[derive(Clone, Debug)]
pub enum CardPosition {
    TableDownturned { stack_index: i8, downturned_index: i8 },
    TableUpturned { stack_index: i8, upturned_index: i8 },
    Ace { suit_index: i8 },
    DrawDeck { deck_index: i8 }
}


#[derive(Debug)]
pub struct GameMove {
    pub from: CardPosition,
    pub to: CardPosition,
}

impl GameMove {
    //Function exists only for debugging
    pub fn debug_move(&self, game: &Game) -> String {
        let mut line2 = String::from("\n");

        let mut cards: Vec<&Card>;
        match *&self.from {
            CardPosition::TableDownturned { stack_index, downturned_index } => {
                assert!(stack_index < 8);

                let stack = &game.table[stack_index as usize];
                line2 += &*format!(" From stack {:?}", stack);

                cards = vec![&stack.downturned[downturned_index as usize]];
            },
            CardPosition::TableUpturned { stack_index, upturned_index } => {
                assert!(stack_index < 7);

                let stack = &game.table[stack_index as usize];
                line2 += &*format!(" From stack {:?}", stack);

                cards = Vec::new();
                for card in &stack.upturned[(upturned_index as usize)..] {
                    cards.push(card);
                }
            },
            CardPosition::Ace { suit_index } => {
                assert!(suit_index < 4);

                unreachable!("There should be no moves FROM Ace");
            },
            CardPosition::DrawDeck { deck_index } => {
                assert!(deck_index < 24);

                cards = vec![&game.draw[deck_index as usize]];
            },
        }

        //Add to 'to'
        match *&self.to {
            CardPosition::TableUpturned { stack_index, upturned_index } => {
                assert!(stack_index < 7);
                let stack = &game.table[stack_index as usize];
                line2 += &*format!("\nTo stack {:?}", stack);

                if let Some(onto_card) = stack.upturned.get((upturned_index-1) as usize) {
                    return format!("Moving {:?} onto [{:?}]\n{}", cards, onto_card, line2);
                }
                return format!("Moving {:?}", cards);
            },
            CardPosition::Ace { suit_index } => {
                return format!("Moving {:?} to ace [{}]\n{}", cards, suit_index, line2);
            },
            _ => {
                return format!("Moving {:?}\n{}", cards, line2);
            },
        }
    }

    pub fn execute(&self, game: &mut Box<Game>) {

        //Fetch & remove from 'from'
        let mut cards: Vec<Card>;

        match *&self.from {
            CardPosition::TableDownturned { stack_index, downturned_index } => {
                assert!(stack_index < 8);

                let stack = &mut game.table[stack_index as usize];
                cards = vec![stack.downturned.remove(downturned_index as usize)];
            },
            CardPosition::TableUpturned { stack_index, upturned_index } => {
                assert!(stack_index < 7);

                let stack = &mut game.table[stack_index as usize];
                cards = stack.upturned.drain((upturned_index as usize)..).collect();
            },
            CardPosition::Ace { suit_index } => {
                assert!(suit_index < 4);

                unreachable!("There should be no moves FROM Ace");
            },
            CardPosition::DrawDeck { deck_index } => {
                assert!(deck_index < 24);

                cards = vec![game.draw.remove(deck_index as usize)];
            },
        }
        println!(" - - Executing move {:?}", cards);

        //Add to 'to'
        match *&self.to {
            CardPosition::TableDownturned { stack_index, .. } => {
                assert!(stack_index < 7);

                unreachable!("There should be no moves TO TableDownturned")
            },
            CardPosition::TableUpturned { stack_index, upturned_index } => {
                assert!(stack_index < 7);
                let stack = &mut game.table[stack_index as usize];
                let upturned_index_usize = upturned_index as usize;

                stack.upturned.splice(upturned_index_usize..upturned_index_usize, cards);

                if let Some(onto_card) = stack.upturned.get((upturned_index-1) as usize) {
                    println!(" - - - Onto [{:?}]", onto_card);
                }
            },
            CardPosition::Ace { suit_index } => {
                assert!(suit_index < 4);

                //Can only move 1 card to the ace (at a time)
                assert_eq!(cards.len(), 1);
                let card = cards.remove(0);

                let ace_stack = &mut game.aces[suit_index as usize];

                //Check the last element's number is our number - 1
                match ace_stack.ace_stack.last() {
                    Some(last) => {
                        assert_eq!(last.number, card.number - 1);
                    },
                    None => (),
                };

                ace_stack.ace_stack.push(card);
            },
            CardPosition::DrawDeck { deck_index } => {
                assert!(deck_index < 28);

                unreachable!("There should be no moves TO DrawDeck")
            },
        }
    }
}