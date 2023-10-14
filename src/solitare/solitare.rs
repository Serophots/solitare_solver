use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::solitare::card::{Card, Suit};
use crate::solitare::state::{CardPosition, GameMove, GameState};

pub struct Game<'a> {
    pub table: [TableStack<'a>; 7],
    pub aces: [AceStack<'a>; 4],
    pub draw: Vec<Card<'a>>,
    pub _phantom: PhantomData<&'a ()>
}

impl<'a> Game<'a> {
    pub fn new() -> Self {
        //We propagate changes through the sub structures like Stack, AceStack, Card which all own their descendent cards
        // -> no master deck - this is duplicate cloned date whose structure/ordering is not manipulated through the above strategy, and is therefore pointless

        //Instead, the deck should be generated randomly here and Cards constructed in here under the ownership of the correct properties of this struct

        let mut deck_integers: Vec<i8> = (0..52).collect();
        deck_integers.shuffle(&mut thread_rng());

        // let deck_integers = vec!(19, 24, 29, 51, 16, 4, 13, 11, 31, 43, 33, 22, 44, 34, 39, 27, 5, 48, 7, 14, 10, 40, 18, 15, 2, 20, 6, 30, 50, 35, 0, 47, 46, 38, 3, 37, 17, 9, 26, 32, 21, 42, 25, 23, 1, 12, 49, 36, 45, 41, 28, 8);

        println!("Created game: {:?}", deck_integers);



        let mut deck_integers_iter = deck_integers.iter();

        let next_card = |deck_integers_iter: &mut slice::Iter<i8>| {
            let card_index = *deck_integers_iter.next().expect("Drew too many cards");
            Card::new(card_index)
        };

        let next_n_cards = |deck_integers_iter: &mut slice::Iter<i8>, n: i8| {
            let mut cards: Vec<Card> = Vec::with_capacity(n as usize);

            for _ in 0..n {
                let card_index = *deck_integers_iter.next().expect("Drew too many cards");
                cards.push(Card::new(card_index));
            }

            cards
        };

        Self {
            table: [
                TableStack {
                    downturned: Vec::new(),
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
                TableStack {
                    downturned: vec![next_card(&mut deck_integers_iter)],
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
                TableStack {
                    downturned: vec![next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter)],
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
                TableStack {
                    downturned: vec![next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter)],
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
                TableStack {
                    downturned: vec![next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter)],
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
                TableStack {
                    downturned: vec![next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter)],
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
                TableStack {
                    downturned: vec![next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter), next_card(&mut deck_integers_iter)],
                    upturned: vec!(next_card(&mut deck_integers_iter))
                },
            ],
            aces: [
                AceStack {ace_stack: Vec::new()},
                AceStack {ace_stack: Vec::new()},
                AceStack {ace_stack: Vec::new()},
                AceStack {ace_stack: Vec::new()}
            ],
            draw: next_n_cards(&mut deck_integers_iter, 24),
            _phantom: PhantomData::default()
        }
    }

    pub fn get_table_flip_moves(&self) -> Vec<GameMove> {
        //Flip downward cards with nothing on top
        let mut moves = Vec::new();

        for (stack_index, table_stack) in self.table.iter().enumerate() {
            let downturned_len = table_stack.downturned.len();
            if table_stack.upturned.len() == 0 && downturned_len != 0 {

                moves.push(GameMove {
                    from: CardPosition::TableDownturned {stack_index: stack_index as i8, downturned_index: (downturned_len-1) as i8},
                    to: CardPosition::TableUpturned { stack_index: stack_index as i8, upturned_index: 0 }
                })

            }
        }

        moves
    }
    pub fn get_table_ace_moves(&self) -> Vec<GameMove> {
        //Move aces up from table
        let mut moves = Vec::new();

        for (stack_index, table_stack) in self.table.iter().enumerate() {
            match table_stack.upturned.last() {
                Some(final_card) => {
                    //For each final card, is it an ace?
                    if final_card.number == 1 {

                        moves.push(GameMove {
                            from: CardPosition::TableUpturned { stack_index: stack_index as i8, upturned_index: (table_stack.upturned.len()-1) as i8 },
                            to: CardPosition::Ace { suit_index: final_card.suit_index }
                        });

                    }
                },
                None => (),
            }
        }

        moves
    }
    pub fn get_draw_ace_moves(&self) -> Vec<GameMove> {
        //Move aces up from deck
        let mut moves = Vec::new();

        for (deck_index, deck_card) in self.draw.iter().enumerate() {

            if deck_card.number == 1 {
                moves.push(GameMove {
                    from: CardPosition::DrawDeck { deck_index: deck_index as i8 },
                    to: CardPosition::Ace { suit_index: deck_card.suit_index }
                })
            }

        }

        moves
    }
    pub fn get_table_king_moves(&self) -> (Vec<GameMove>, i8) {
        //Move kings from table to empty spot
        let mut moves = Vec::new();
        let mut table_queuing_kings = Vec::new();
        let mut draw_queuing_kings = Vec::new();

        //Aggregate table queuing kings
        for (stack_index, table_stack) in self.table.iter().enumerate() {
            //Exclude moves from stacks with no downturned cards
            if table_stack.downturned.len() != 0 {
                match table_stack.upturned.first() {
                    Some(first_card) => {

                        //For each root upturned card, is it a king?
                        if first_card.number == 13 {

                            table_queuing_kings.push(CardPosition::TableUpturned {
                                stack_index: stack_index as i8,
                                upturned_index: 0
                            })


                        }
                    },
                    None => ()
                }
            }
        }

        //Aggregate draw queuing kings
        for (draw_index, draw_card) in self.draw.iter().enumerate() {
            if draw_card.number == 13 {

                draw_queuing_kings.push(CardPosition::DrawDeck {
                    deck_index: draw_index as i8
                });

            }
        }

        let vacant_kings: i8 = (table_queuing_kings.len() + draw_queuing_kings.len()) as i8;

        //Find a vacant table-stack - Each cycle only 1 move is ever executed at most - therefore we can consider that all of the potential kings move to the same vacant spot
        let mut vacant_stack_index: i8 = 0;

        for table_stack in &self.table {
            if table_stack.upturned.len() == 0
                && table_stack.downturned.len() == 0
            {
                break
            }
            vacant_stack_index += 1;
        }

        //Add table moves
        if vacant_stack_index < 7 {
            for card_position in table_queuing_kings {
                moves.push(GameMove {
                    from: card_position,
                    to: CardPosition::TableUpturned { stack_index: vacant_stack_index, upturned_index: 0 }
                })
            }
        }

        (moves, vacant_kings)
    }
    pub fn get_ace_stack_moves(&self) -> Vec<GameMove> {
        //Move from either table or draw onto aces
        let mut moves = Vec::new();

        //Gather what number each of the ace stacks are at
        let mut aces: HashMap<Suit, i8> = HashMap::new();
        for ace_stack in &self.aces {

            match ace_stack.ace_stack.last() {
                Some(final_ace) => {
                    aces.insert(final_ace.suit, final_ace.number);
                },
                None => (),
            }

        }


        //Table
        for (stack_index, table_stack) in self.table.iter().enumerate() {
            match table_stack.upturned.last() {
                Some(final_card) => {
                    if let Some(latest_ace_num) = aces.get(&final_card.suit) {
                        if final_card.number == latest_ace_num + 1 {

                            moves.push(GameMove {
                                from: CardPosition::TableUpturned { stack_index: stack_index as i8, upturned_index: (table_stack.upturned.len()-1) as i8 },
                                to: CardPosition::Ace { suit_index: final_card.suit_index }
                            });

                        }
                    }
                },
                None => (),
            }
        }

        //Draw
        for (deck_index, draw_card) in self.draw.iter().enumerate() {
            if let Some(latest_ace_num) = aces.get(&draw_card.suit) {
                if draw_card.number == latest_ace_num + 1 {
                    moves.push(GameMove {
                        from: CardPosition::DrawDeck { deck_index: deck_index as i8 },
                        to: CardPosition::Ace { suit_index: draw_card.suit_index }
                    })
                }
            }
        }

        moves
    }
    pub fn get_table_moves(&self, final_cards: &Vec<(&Card, CardPosition)>) -> Vec<GameMove> {
        //Moves within the table between stacks
        let mut moves = Vec::new();

        //Find the moves
        for (stack_index, table_stack) in self.table.iter().enumerate() {
            for (upturned_index, upturned_card) in table_stack.upturned.iter().enumerate() {

                //For each upturned card, compare it against the final cards of all the stacks
                for (compare_card, compare_position) in final_cards {

                    //If they're different colour, consecutive numbers, its a move
                    if !upturned_card.suit.same_color(compare_card.suit) {
                        if compare_card.number == upturned_card.number + 1 {

                            moves.push(GameMove {
                                from: CardPosition::TableUpturned { stack_index: stack_index as i8, upturned_index: upturned_index as i8 },
                                to: compare_position.clone()
                            });

                        }
                    }
                }
            }
        }

        moves
    }
    pub fn get_deck_moves(&self, final_cards: &Vec<(&Card, CardPosition)>) -> Vec<GameMove> {
        //Moves from the deck to the table
        let mut moves = Vec::new();

        //Find the moves
        for (draw_index, draw_card) in self.draw.iter().enumerate() {

            //For each draw card, compare it against the final cards of all the stacks
            for (compare_card, compare_position) in final_cards {


                //If they're different colour, consecutive numbers, its a move
                if !draw_card.suit.same_color(compare_card.suit) {
                    if compare_card.number == draw_card.number + 1 {

                        moves.push(GameMove {
                            from: CardPosition::DrawDeck { deck_index: draw_index as i8 },
                            to: compare_position.clone()
                        });

                    }
                }

            }
        }

        moves
    }


    pub fn get_game_state(&self) -> GameState {

        //Gather the last upturned cards of each stack to potentially move a stack onto
        let mut final_table_stack_cards: Vec<(&Card, CardPosition)> = Vec::with_capacity(4); //There is always at least 4 stacks
        for (stack_index, table_stack) in self.table.iter().enumerate() {
            match table_stack.upturned.last() {
                Some(final_card) => {

                    final_table_stack_cards.push((
                        final_card,
                        CardPosition::TableUpturned {
                            stack_index: stack_index as i8,
                            upturned_index: (table_stack.upturned.len()) as i8 //Dont -1 because this is the position  we're placing onto, after the current card
                        }
                    ))


                },
                None => (),
            }
        }

        //Run all the functions
        let (table_king_moves, queuing_kings) = self.get_table_king_moves();

        GameState {
            table_flip_moves: self.get_table_flip_moves(),
            table_ace_moves: self.get_table_ace_moves(),
            draw_ace_moves: self.get_draw_ace_moves(),
            table_king_moves,
            ace_stack_moves: self.get_ace_stack_moves(),
            table_moves: self.get_table_moves(&final_table_stack_cards),
            deck_moves: self.get_deck_moves(&final_table_stack_cards),


            aces: &self.aces,
            // final_table_stack_cards,
            queuing_kings
        }
    }

}




#[derive(Debug)]
pub struct AceStack<'a> {
    pub ace_stack: Vec<Card<'a>>
}
impl<'a> AceStack<'a> {
    pub fn is_full(&self) -> bool { self.ace_stack.len() >= 13 }
}



#[derive(Debug)]
pub struct TableStack<'a> {
    pub downturned: Vec<Card<'a>>,
    pub upturned: Vec<Card<'a>>
}
