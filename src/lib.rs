/// The rank of a playing card, ordered from lowest (Deuce) to highest (Ace).
///
/// The discriminant value is used to access the `PRIMES` table as an index,
/// compute the card's face value, and activate the one-hot bit position
/// in the upper 16 bits of the Cactus Kev encoding.
#[repr(u8)]
#[derive(PartialEq, Debug, Clone, Copy)]
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
    pub fn from_char(value: char) -> Option<Self> {
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
    use super::*;
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
#[derive(PartialEq, Debug, Clone, Copy)]
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
    pub fn from_char(value: char) -> Option<Self> {
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
