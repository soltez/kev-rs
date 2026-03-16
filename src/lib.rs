//! # kev-rs
//!
//! A Rust implementation of Cactus Kev's 32-bit card integer encoding.
//!
//! Each playing card is represented as a `u32` with rank, suit, prime, and
//! one-hot rank bits packed into a single integer. This layout enables
//! efficient hand evaluation using bitwise operations and prime-product
//! lookup tables.
//!
//! ## Bit layout
//!
//! ```text
//! +--------+--------+--------+--------+
//! |xxxbbbbb|bbbbbbbb|cdhsrrrr|xxpppppp|
//! +--------+--------+--------+--------+
//!
//! Bits 28–16: b = one-hot rank bit
//! Bits 15–12: cdhs = one-hot suit nibble (c=clubs, d=diamonds, h=hearts, s=spades)
//! Bits 11– 8: r = rank index (deuce=0, trey=1, ..., ace=12)
//! Bits  5– 0: p = rank prime  (deuce=2, trey=3, ..., ace=41)
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use kev::CardInt;
//!
//! let ace_of_spades = CardInt::CardAs;
//! let ace_of_clubs = CardInt::CardAc;
//! let king_of_clubs = CardInt::new("Kc").unwrap();
//! assert_eq!(ace_of_spades.rank(), ace_of_clubs.rank());
//! assert_eq!(ace_of_clubs.suit(), king_of_clubs.suit());
//! ```

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

pub mod hand;

/// The rank of a playing card, ordered from lowest (Deuce) to highest (Ace).
///
/// The discriminant value is used to access the `PRIMES` table as an index,
/// compute the card's face value, and activate the one-hot bit position
/// in the upper 16 bits of the Cactus Kev encoding.
#[repr(u8)]
#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum Rank {
    /// Two (2)
    Deuce = 0,
    /// Three (3)
    Trey = 1,
    /// Four (4)
    Four = 2,
    /// Five (5)
    Five = 3,
    /// Six (6)
    Six = 4,
    /// Seven (7)
    Seven = 5,
    /// Eight (8)
    Eight = 6,
    /// Nine (9)
    Nine = 7,
    /// Ten (T)
    Ten = 8,
    /// Jack (J)
    Jack = 9,
    /// Queen (Q)
    Queen = 10,
    /// King (K)
    King = 11,
    /// Ace (A)
    Ace = 12,
}

impl Rank {
    /// Parses a single character into a `Rank`.
    ///
    /// Accepts both upper- and lowercase letters (`A`/`a` through `2`), plus
    /// `T`/`t` for Ten. Returns `None` for any unrecognised character.
    fn from_char(value: char) -> Option<Self> {
        match value {
            'A' | 'a' => Some(Rank::Ace),
            'K' | 'k' => Some(Rank::King),
            'Q' | 'q' => Some(Rank::Queen),
            'J' | 'j' => Some(Rank::Jack),
            'T' | 't' => Some(Rank::Ten),
            '9' => Some(Rank::Nine),
            '8' => Some(Rank::Eight),
            '7' => Some(Rank::Seven),
            '6' => Some(Rank::Six),
            '5' => Some(Rank::Five),
            '4' => Some(Rank::Four),
            '3' => Some(Rank::Trey),
            '2' => Some(Rank::Deuce),
            _ => None,
        }
    }
}

#[cfg(test)]
mod rank_tests {
    use super::Rank;
    use rstest::rstest;

    #[rstest]
    #[case('A', Some(Rank::Ace))]
    #[case('a', Some(Rank::Ace))]
    #[case('K', Some(Rank::King))]
    #[case('k', Some(Rank::King))]
    #[case('Q', Some(Rank::Queen))]
    #[case('q', Some(Rank::Queen))]
    #[case('J', Some(Rank::Jack))]
    #[case('j', Some(Rank::Jack))]
    #[case('T', Some(Rank::Ten))]
    #[case('t', Some(Rank::Ten))]
    #[case('9', Some(Rank::Nine))]
    #[case('8', Some(Rank::Eight))]
    #[case('7', Some(Rank::Seven))]
    #[case('6', Some(Rank::Six))]
    #[case('5', Some(Rank::Five))]
    #[case('4', Some(Rank::Four))]
    #[case('3', Some(Rank::Trey))]
    #[case('2', Some(Rank::Deuce))]
    #[case('1', None)]
    fn from_char(#[case] input: char, #[case] expected: Option<Rank>) {
        assert_eq!(Rank::from_char(input), expected)
    }
}

/// The suit of a playing card.
///
/// The discriminant is a one-hot nibble stored in bits 12–15 of the Cactus Kev
/// card integer, so flush detection can be tested with a
/// single bitwise AND.
#[repr(u8)]
#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum Suit {
    /// Spades (♠)
    Spade = 0x1,
    /// Hearts (♥)
    Heart = 0x2,
    /// Diamonds (♦)
    Diamond = 0x4,
    /// Clubs (♣)
    Club = 0x8,
}

impl Suit {
    /// Parses a single character into a `Suit`.
    ///
    /// Accepts Unicode suit symbols (`♠ ♤ ♥ ♡ ♦ ♢ ♣ ♧`) as well as ASCII
    /// letters (`S`/`s`, `H`/`h`, `D`/`d`, `C`/`c`). Returns `None` for any
    /// unrecognised character.
    fn from_char(value: char) -> Option<Self> {
        match value {
            '♤' | '♠' | 'S' | 's' => Some(Suit::Spade),
            '♡' | '♥' | 'H' | 'h' => Some(Suit::Heart),
            '♢' | '♦' | 'D' | 'd' => Some(Suit::Diamond),
            '♧' | '♣' | 'C' | 'c' => Some(Suit::Club),
            _ => None,
        }
    }
}

#[cfg(test)]
mod suit_tests {
    use super::Suit;
    use rstest::rstest;

    #[rstest]
    #[case('♤', Some(Suit::Spade))]
    #[case('♠', Some(Suit::Spade))]
    #[case('S', Some(Suit::Spade))]
    #[case('s', Some(Suit::Spade))]
    #[case('♡', Some(Suit::Heart))]
    #[case('♥', Some(Suit::Heart))]
    #[case('H', Some(Suit::Heart))]
    #[case('h', Some(Suit::Heart))]
    #[case('♢', Some(Suit::Diamond))]
    #[case('♦', Some(Suit::Diamond))]
    #[case('D', Some(Suit::Diamond))]
    #[case('d', Some(Suit::Diamond))]
    #[case('♧', Some(Suit::Club))]
    #[case('♣', Some(Suit::Club))]
    #[case('C', Some(Suit::Club))]
    #[case('c', Some(Suit::Club))]
    #[case('z', None)]
    fn from_char(#[case] input: char, #[case] expected: Option<Suit>) {
        assert_eq!(Suit::from_char(input), expected)
    }
}

/// A playing card represented as a 32-bit integer using the Cactus Kev encoding.
///
/// Each variant encodes a unique (rank, suit) pair in a single `u32` with the
/// following bit layout:
///
/// ```text
/// An integer is made up of four bytes.  The high-order bytes are
/// used to hold the rank bit pattern, whereas the low-order bytes
/// hold the suit/rank/prime value of the card.
///
/// +--------+--------+--------+--------+
/// |xxxbbbbb|bbbbbbbb|cdhsrrrr|xxpppppp|
/// +--------+--------+--------+--------+
///
/// Bits 28–16: b = bit turned on depending on rank of card
/// Bits 15–12: cdhs = suit of card (bit turned on based on suit of card)
/// Bits 11– 8: r = rank of card (deuce=0,trey=1,four=2,five=3,...,ace=12)
/// Bits  5– 0: p = prime number of rank (deuce=2,trey=3,four=5,...,ace=41)
/// ```
///
/// Variants are named `Card<Rank><Suit>` where rank uses its conventional
/// character (`A K Q J T 9 ... 2`) and suit uses its initial (`s h d c`).
#[repr(u32)]
#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum CardInt {
    CardAs = 0b0001_0000_0000_0000_0001_1100_0010_1001,
    CardKs = 0b0000_1000_0000_0000_0001_1011_0010_0101,
    CardQs = 0b0000_0100_0000_0000_0001_1010_0001_1111,
    CardJs = 0b0000_0010_0000_0000_0001_1001_0001_1101,
    CardTs = 0b0000_0001_0000_0000_0001_1000_0001_0111,
    Card9s = 0b0000_0000_1000_0000_0001_0111_0001_0011,
    Card8s = 0b0000_0000_0100_0000_0001_0110_0001_0001,
    Card7s = 0b0000_0000_0010_0000_0001_0101_0000_1101,
    Card6s = 0b0000_0000_0001_0000_0001_0100_0000_1011,
    Card5s = 0b0000_0000_0000_1000_0001_0011_0000_0111,
    Card4s = 0b0000_0000_0000_0100_0001_0010_0000_0101,
    Card3s = 0b0000_0000_0000_0010_0001_0001_0000_0011,
    Card2s = 0b0000_0000_0000_0001_0001_0000_0000_0010,
    CardAh = 0b0001_0000_0000_0000_0010_1100_0010_1001,
    CardKh = 0b0000_1000_0000_0000_0010_1011_0010_0101,
    CardQh = 0b0000_0100_0000_0000_0010_1010_0001_1111,
    CardJh = 0b0000_0010_0000_0000_0010_1001_0001_1101,
    CardTh = 0b0000_0001_0000_0000_0010_1000_0001_0111,
    Card9h = 0b0000_0000_1000_0000_0010_0111_0001_0011,
    Card8h = 0b0000_0000_0100_0000_0010_0110_0001_0001,
    Card7h = 0b0000_0000_0010_0000_0010_0101_0000_1101,
    Card6h = 0b0000_0000_0001_0000_0010_0100_0000_1011,
    Card5h = 0b0000_0000_0000_1000_0010_0011_0000_0111,
    Card4h = 0b0000_0000_0000_0100_0010_0010_0000_0101,
    Card3h = 0b0000_0000_0000_0010_0010_0001_0000_0011,
    Card2h = 0b0000_0000_0000_0001_0010_0000_0000_0010,
    CardAd = 0b0001_0000_0000_0000_0100_1100_0010_1001,
    CardKd = 0b0000_1000_0000_0000_0100_1011_0010_0101,
    CardQd = 0b0000_0100_0000_0000_0100_1010_0001_1111,
    CardJd = 0b0000_0010_0000_0000_0100_1001_0001_1101,
    CardTd = 0b0000_0001_0000_0000_0100_1000_0001_0111,
    Card9d = 0b0000_0000_1000_0000_0100_0111_0001_0011,
    Card8d = 0b0000_0000_0100_0000_0100_0110_0001_0001,
    Card7d = 0b0000_0000_0010_0000_0100_0101_0000_1101,
    Card6d = 0b0000_0000_0001_0000_0100_0100_0000_1011,
    Card5d = 0b0000_0000_0000_1000_0100_0011_0000_0111,
    Card4d = 0b0000_0000_0000_0100_0100_0010_0000_0101,
    Card3d = 0b0000_0000_0000_0010_0100_0001_0000_0011,
    Card2d = 0b0000_0000_0000_0001_0100_0000_0000_0010,
    CardAc = 0b0001_0000_0000_0000_1000_1100_0010_1001,
    CardKc = 0b0000_1000_0000_0000_1000_1011_0010_0101,
    CardQc = 0b0000_0100_0000_0000_1000_1010_0001_1111,
    CardJc = 0b0000_0010_0000_0000_1000_1001_0001_1101,
    CardTc = 0b0000_0001_0000_0000_1000_1000_0001_0111,
    Card9c = 0b0000_0000_1000_0000_1000_0111_0001_0011,
    Card8c = 0b0000_0000_0100_0000_1000_0110_0001_0001,
    Card7c = 0b0000_0000_0010_0000_1000_0101_0000_1101,
    Card6c = 0b0000_0000_0001_0000_1000_0100_0000_1011,
    Card5c = 0b0000_0000_0000_1000_1000_0011_0000_0111,
    Card4c = 0b0000_0000_0000_0100_1000_0010_0000_0101,
    Card3c = 0b0000_0000_0000_0010_1000_0001_0000_0011,
    Card2c = 0b0000_0000_0000_0001_1000_0000_0000_0010,
}

#[derive(Debug, Error)]
pub enum CardError {
    #[error("invalid rank: '{0}'")]
    InvalidRank(char),

    #[error("invalid suit: '{0}'")]
    InvalidSuit(char),

    #[error("invalid input: '{0}'")]
    InvalidInput(String),
}

impl CardInt {
    /// Constructs a `CardInt` from a two-character string such as `"As"` or `"Td"`.
    ///
    /// The first character is parsed as a [`Rank`] via `Rank::from_char` and
    /// the second as a [`Suit`] via `Suit::from_char`. Returns [`CardError`] if
    /// either character is unrecognised or the string is not exactly two
    /// characters long.
    pub fn new(s: &str) -> Result<Self, CardError> {
        let invalid = || CardError::InvalidInput(s.to_string());
        let mut chars = s.chars();
        let rank: Rank = chars
            .next()
            .ok_or_else(invalid)
            .and_then(|c| Rank::from_char(c).ok_or(CardError::InvalidRank(c)))?;
        let suit: Suit = chars
            .next()
            .ok_or_else(invalid)
            .and_then(|c| Suit::from_char(c).ok_or(CardError::InvalidSuit(c)))?;
        chars.next().map_or(Ok(()), |_| Err(invalid()))?;
        Ok(Self::_new(&rank, &suit))
    }

    /// Constructs a `CardInt` from a [`Rank`] and [`Suit`] by computing the
    /// Cactus Kev bit pattern directly.
    fn _new(rank: &Rank, suit: &Suit) -> CardInt {
        let prime: u32 = match rank {
            Rank::Deuce => 2,
            Rank::Trey => 3,
            Rank::Four => 5,
            Rank::Five => 7,
            Rank::Six => 11,
            Rank::Seven => 13,
            Rank::Eight => 17,
            Rank::Nine => 19,
            Rank::Ten => 23,
            Rank::Jack => 29,
            Rank::Queen => 31,
            Rank::King => 37,
            Rank::Ace => 41,
        };
        let rank_nib: u32 = (*rank as u32) << 8;
        let suit_nib: u32 = (*suit as u32) << 12;
        let onehot: u32 = 1 << (*rank as u32) << 16;
        CardInt::from_u32(prime | rank_nib | suit_nib | onehot).unwrap()
    }

    /// Extracts the [`Rank`] from this card's face-value field (bits 8–11).
    pub fn rank(&self) -> Rank {
        Rank::from_u8((*self as u32 >> 8 & 0xF) as u8).unwrap()
    }

    /// Extracts the [`Suit`] from this card's suit nibble (bits 12–15).
    pub fn suit(&self) -> Suit {
        Suit::from_u8((*self as u32 >> 12 & 0xF) as u8).unwrap()
    }
}

#[cfg(test)]
mod card_integer_tests {
    use super::{CardInt, FromPrimitive, Rank, Suit};
    use rstest::rstest;

    #[rstest]
    #[case(0b00001000_00000000_01001011_00100101, CardInt::CardKd)]
    #[case(0b00000000_00001000_00010011_00000111, CardInt::Card5s)]
    #[case(0b00000010_00000000_10001001_00011101, CardInt::CardJc)]
    #[case(0b00000100_00000000_10001010_00011111, CardInt::CardQc)]
    fn bit_pattern_example(#[case] input: u32, #[case] expected: CardInt) {
        assert_eq!(CardInt::from_u32(input), Some(expected));
    }

    #[rstest]
    #[case(Rank::Ace, Suit::Spade, CardInt::CardAs)]
    #[case(Rank::King, Suit::Spade, CardInt::CardKs)]
    #[case(Rank::Queen, Suit::Spade, CardInt::CardQs)]
    #[case(Rank::Jack, Suit::Spade, CardInt::CardJs)]
    #[case(Rank::Ten, Suit::Spade, CardInt::CardTs)]
    #[case(Rank::Nine, Suit::Spade, CardInt::Card9s)]
    #[case(Rank::Eight, Suit::Spade, CardInt::Card8s)]
    #[case(Rank::Seven, Suit::Spade, CardInt::Card7s)]
    #[case(Rank::Six, Suit::Spade, CardInt::Card6s)]
    #[case(Rank::Five, Suit::Spade, CardInt::Card5s)]
    #[case(Rank::Four, Suit::Spade, CardInt::Card4s)]
    #[case(Rank::Trey, Suit::Spade, CardInt::Card3s)]
    #[case(Rank::Deuce, Suit::Spade, CardInt::Card2s)]
    #[case(Rank::Ace, Suit::Heart, CardInt::CardAh)]
    #[case(Rank::King, Suit::Heart, CardInt::CardKh)]
    #[case(Rank::Queen, Suit::Heart, CardInt::CardQh)]
    #[case(Rank::Jack, Suit::Heart, CardInt::CardJh)]
    #[case(Rank::Ten, Suit::Heart, CardInt::CardTh)]
    #[case(Rank::Nine, Suit::Heart, CardInt::Card9h)]
    #[case(Rank::Eight, Suit::Heart, CardInt::Card8h)]
    #[case(Rank::Seven, Suit::Heart, CardInt::Card7h)]
    #[case(Rank::Six, Suit::Heart, CardInt::Card6h)]
    #[case(Rank::Five, Suit::Heart, CardInt::Card5h)]
    #[case(Rank::Four, Suit::Heart, CardInt::Card4h)]
    #[case(Rank::Trey, Suit::Heart, CardInt::Card3h)]
    #[case(Rank::Deuce, Suit::Heart, CardInt::Card2h)]
    #[case(Rank::Ace, Suit::Diamond, CardInt::CardAd)]
    #[case(Rank::King, Suit::Diamond, CardInt::CardKd)]
    #[case(Rank::Queen, Suit::Diamond, CardInt::CardQd)]
    #[case(Rank::Jack, Suit::Diamond, CardInt::CardJd)]
    #[case(Rank::Ten, Suit::Diamond, CardInt::CardTd)]
    #[case(Rank::Nine, Suit::Diamond, CardInt::Card9d)]
    #[case(Rank::Eight, Suit::Diamond, CardInt::Card8d)]
    #[case(Rank::Seven, Suit::Diamond, CardInt::Card7d)]
    #[case(Rank::Six, Suit::Diamond, CardInt::Card6d)]
    #[case(Rank::Five, Suit::Diamond, CardInt::Card5d)]
    #[case(Rank::Four, Suit::Diamond, CardInt::Card4d)]
    #[case(Rank::Trey, Suit::Diamond, CardInt::Card3d)]
    #[case(Rank::Deuce, Suit::Diamond, CardInt::Card2d)]
    #[case(Rank::Ace, Suit::Club, CardInt::CardAc)]
    #[case(Rank::King, Suit::Club, CardInt::CardKc)]
    #[case(Rank::Queen, Suit::Club, CardInt::CardQc)]
    #[case(Rank::Jack, Suit::Club, CardInt::CardJc)]
    #[case(Rank::Ten, Suit::Club, CardInt::CardTc)]
    #[case(Rank::Nine, Suit::Club, CardInt::Card9c)]
    #[case(Rank::Eight, Suit::Club, CardInt::Card8c)]
    #[case(Rank::Seven, Suit::Club, CardInt::Card7c)]
    #[case(Rank::Six, Suit::Club, CardInt::Card6c)]
    #[case(Rank::Five, Suit::Club, CardInt::Card5c)]
    #[case(Rank::Four, Suit::Club, CardInt::Card4c)]
    #[case(Rank::Trey, Suit::Club, CardInt::Card3c)]
    #[case(Rank::Deuce, Suit::Club, CardInt::Card2c)]
    fn binary_literal_integrity(#[case] rank: Rank, #[case] suit: Suit, #[case] card: CardInt) {
        assert_eq!(CardInt::_new(&rank, &suit), card);
        assert_eq!(card.rank(), rank);
        assert_eq!(card.suit(), suit);
    }

    #[rstest]
    #[case("As", CardInt::CardAs)]
    #[case("Ks", CardInt::CardKs)]
    #[case("Qs", CardInt::CardQs)]
    #[case("Js", CardInt::CardJs)]
    #[case("Ts", CardInt::CardTs)]
    #[case("9s", CardInt::Card9s)]
    #[case("8s", CardInt::Card8s)]
    #[case("7s", CardInt::Card7s)]
    #[case("6s", CardInt::Card6s)]
    #[case("5s", CardInt::Card5s)]
    #[case("4s", CardInt::Card4s)]
    #[case("3s", CardInt::Card3s)]
    #[case("2s", CardInt::Card2s)]
    #[case("Ah", CardInt::CardAh)]
    #[case("Kh", CardInt::CardKh)]
    #[case("Qh", CardInt::CardQh)]
    #[case("Jh", CardInt::CardJh)]
    #[case("Th", CardInt::CardTh)]
    #[case("9h", CardInt::Card9h)]
    #[case("8h", CardInt::Card8h)]
    #[case("7h", CardInt::Card7h)]
    #[case("6h", CardInt::Card6h)]
    #[case("5h", CardInt::Card5h)]
    #[case("4h", CardInt::Card4h)]
    #[case("3h", CardInt::Card3h)]
    #[case("2h", CardInt::Card2h)]
    #[case("Ad", CardInt::CardAd)]
    #[case("Kd", CardInt::CardKd)]
    #[case("Qd", CardInt::CardQd)]
    #[case("Jd", CardInt::CardJd)]
    #[case("Td", CardInt::CardTd)]
    #[case("9d", CardInt::Card9d)]
    #[case("8d", CardInt::Card8d)]
    #[case("7d", CardInt::Card7d)]
    #[case("6d", CardInt::Card6d)]
    #[case("5d", CardInt::Card5d)]
    #[case("4d", CardInt::Card4d)]
    #[case("3d", CardInt::Card3d)]
    #[case("2d", CardInt::Card2d)]
    #[case("Ac", CardInt::CardAc)]
    #[case("Kc", CardInt::CardKc)]
    #[case("Qc", CardInt::CardQc)]
    #[case("Jc", CardInt::CardJc)]
    #[case("Tc", CardInt::CardTc)]
    #[case("9c", CardInt::Card9c)]
    #[case("8c", CardInt::Card8c)]
    #[case("7c", CardInt::Card7c)]
    #[case("6c", CardInt::Card6c)]
    #[case("5c", CardInt::Card5c)]
    #[case("4c", CardInt::Card4c)]
    #[case("3c", CardInt::Card3c)]
    #[case("2c", CardInt::Card2c)]
    fn new(#[case] input: &str, #[case] expected: CardInt) {
        assert_eq!(CardInt::new(input).ok(), Some(expected));
    }

    #[rstest]
    #[case("AsKs")]
    #[case("K")]
    #[case("")]
    fn new_invalid_input(#[case] input: &str) {
        let actual = CardInt::new(input);
        let expect = format!("invalid input: '{}'", input);
        assert_eq!(actual.unwrap_err().to_string(), expect);
    }

    #[rstest]
    #[case("Xc")]
    #[case(" D")]
    #[case("x")]
    fn new_invalid_rank(#[case] input: &str) {
        let actual = CardInt::new(input);
        assert!(actual.unwrap_err().to_string().starts_with("invalid rank"));
    }

    #[rstest]
    #[case("ax")]
    #[case("2 ")]
    #[case("jx2")]
    fn new_invalid_suit(#[case] input: &str) {
        let actual = CardInt::new(input);
        assert!(actual.unwrap_err().to_string().starts_with("invalid suit"));
    }
}
