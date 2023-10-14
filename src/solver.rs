use crate::solitare::solitare::Game;
use crate::solitare::state::{CardPosition, GameFinalState, GameMove, GameState};

pub fn solve(mut game: Box<Game>) {
    'solver: loop {
        let game_state = game.get_game_state();

        match game_state.get_final_state() {

            GameFinalState::UNFINISHED => {
                // println!("game_state: {:?}", game_state);

                //Upturn any downward facing cards with nothing on top of them
                match game_state.table_flip_moves.iter().next() {
                    Some(table_flip_move) => {
                        println!(" - Table flip move {:?}", table_flip_move);

                        table_flip_move.execute(&mut game);
                        continue 'solver
                    },
                    None => (),
                }

                //Put up any aces from table
                match game_state.table_ace_moves.iter().next() {
                    Some(table_ace_move) => {
                        println!(" - Table ace move {:?}", table_ace_move);

                        table_ace_move.execute(&mut game);
                        continue 'solver
                    },
                    None => (),
                }

                //Put up any aces from deck
                match game_state.draw_ace_moves.iter().next() {
                    Some(draw_ace_move) => {
                        println!(" - Making draw ace move {:?}", draw_ace_move);

                        draw_ace_move.execute(&mut game);
                        continue 'solver
                    },
                    None => (),
                }

                //Put up any kings from table
                match game_state.table_king_moves.iter().next() {
                    Some(table_king_move) => {
                        println!(" - Making table king move {:?}", table_king_move);

                        table_king_move.execute(&mut game);
                        continue 'solver
                    },
                    None => ()
                }

                //Consider positivity of putting up cards onto aces
                for ace_stack_move in &game_state.ace_stack_moves {
                    let (do_move, move_reason) = is_move_positive(ace_stack_move, &game, &game_state);

                    if do_move {
                        println!(" - Making ace stack move {:?}\n{}", ace_stack_move, move_reason);

                        ace_stack_move.execute(&mut game);
                        continue 'solver
                    }
                }

                //Consider positivity of moves within table
                for table_move in &game_state.table_moves {
                    let (do_move, move_reason) = is_move_positive(table_move, &game, &game_state);
                    if do_move {
                        println!(" - Making table move {:?}\n{}", table_move, move_reason);

                        table_move.execute(&mut game);
                        continue 'solver
                    }
                }

                //Consider positivity of moves from deck to table
                for deck_move in &game_state.deck_moves {
                    let (do_move, move_reason) = is_move_positive(deck_move, &game, &game_state);
                    if do_move {
                        println!(" - Making deck move {:?}\n{}", deck_move, move_reason);

                        deck_move.execute(&mut game);
                        continue 'solver
                    }
                }


                //Made no moves - enter debugging state where we print every potential move and why we wont make it
                println!("\n\nDEBUG STATE\n\n");
                for game_move in game_state.get_all_moves_youch() {
                    println!("Move {:?}\npositivity {:?}\n\n", game_move.debug_move(&game), is_move_positive(game_move, &game, &game_state))
                }

                println!("Made no moves {:?}", game_state);
                break 'solver

                //TODO: Complete moves that aren't positive as desperate last attempt
            },
            GameFinalState::LOST => {

                println!("LOST");

                break 'solver
            },
            GameFinalState::WON => {

                println!("WON");

                break 'solver
            }
        }
    }
}

fn is_move_positive<'a>(game_move: &GameMove, game: &Game, game_state: &GameState) -> (bool, &'a str) {
    //A move is positive if:
    // - It creates space for a queuing king DONE
    // - It reveals a card beneath DONE
    // - It enables a positive move //TODO: Create positivity priority in this is only half as positive as others
    
    //Consider that we may be moving multiple cards in a single move
    match game_move.from {
        CardPosition::TableUpturned { stack_index, upturned_index } => {

            //MOVING FROM TABLE UPTURNED DOES NOT CHECK IF IT ENABLES A MOVE

            let stack = &game.table[stack_index as usize];

            if upturned_index == 0 {
                if stack.downturned.len() == 0 {
                    if game_state.queuing_kings > 0 {
                        //We create space for a king - do the move

                        return (true, " - - Positive move : creates space for king");
                    } else {
                        return (false, " - - No queuing kings")
                    }
                }else{
                    //We reveal a card beneath - do the move

                    return (true, " - - Positive move : reveals a card");
                }
            }
        },
        CardPosition::TableDownturned { .. } => {
            unreachable!()
        },
        CardPosition::DrawDeck { deck_index } => {
            let card = &game.draw[deck_index as usize];

            //We can only ever deem a move positive via creating a second move when we're adding an external card to the table
            //This also guarantees that we're only moving a single card

            match game_move.to {
                CardPosition::TableUpturned { stack_index, upturned_index } => {
                    let to_stack = &game.table[stack_index as usize];

                    //Check we're adding to the bottom of a stack
                    if to_stack.upturned.len() as i8 == upturned_index {

                        //Check every other root upturned card
                        for table_stack in &game.table {
                            if let Some(root_card) = table_stack.upturned.first() {

                                if !card.suit.same_color(root_card.suit) {
                                    if root_card.number == card.number + 1 {
                                        //This move at least enables a further move

                                        return (true, " - - Positive move : enables a move");
                                    }
                                }
                            }
                        }
                        return (false, "doesn't enable a move")
                    }
                },
                _ => ()
            }
        },
        CardPosition::Ace { .. } => {
            unreachable!()
        }
    }

    return (false, "")
}