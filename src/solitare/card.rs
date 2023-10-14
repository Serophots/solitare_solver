use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Clone)] //Not copy.
pub struct Card<'a> {
    pub suit: Suit,
    pub suit_index: i8,
    pub number: i8,
    pub next_card: Option<Box<Card<'a>>>,
    pub _phantom: PhantomData<&'a ()>
}

impl<'d> Card<'d> {
    pub fn new(card_index: i8) -> Self {
        Self {
            suit: Suit::from_index(card_index / 13),
            suit_index: card_index / 13,
            number: card_index.rem_euclid(13) + 1,
            next_card: None,
            _phantom: PhantomData::default()
        }
    }
}

impl<'a> Debug for Card<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}", self.suit, self.number)
    }
}


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Suit {
    HEARTS,
    DIAMONDS,
    CLUBS,
    SPADES
}

impl Suit {
    pub fn from_index(index: i8) -> Self {
        //Ouch
        if index == 0 {
            Self::HEARTS
        }else if index == 1 {
            Self::DIAMONDS
        }else if index == 2 {
            Self::CLUBS
        }else {
            Self::SPADES
        }
    }

    pub fn same_color(&self, other: Suit) -> bool {
        return if *self == Self::HEARTS || *self == Self::DIAMONDS {
            other == Self::HEARTS || other == Self::DIAMONDS
        } else {
            other == Self::CLUBS || other == Self::SPADES
        }
    }
}