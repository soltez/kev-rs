//! Bitwise and arithmetic operations over a slice of [`CardInt`] values.
//!
//! This module provides the three primitive hand-evaluation building blocks
//! used by the Cactus Kev poker hand evaluator:
//!
//! | Function | Bits used | Purpose |
//! |---|---|---|
//! | [`suit_bitwise_and`] | 12–15 (suit nibble) | Flush detection |
//! | [`rank_bitwise_or`]  | 16–28 (rank one-hot) | Straight / rank-presence mask |
//! | [`prime_product`]    | 0–5 (prime byte)     | Unique rank-multiset key |
//!
//! All three functions accept any non-empty slice of [`CardInt`] cards, making
//! them suitable for both 5-card hands and arbitrary board/hole subsets.
//!
//! # Examples
//!
//! ```
//! use kev::CardInt;
//! use kev::hand::{suit_bitwise_and, rank_bitwise_or, prime_product};
//!
//! let royal_flush = &[
//!     CardInt::CardAs, CardInt::CardKs, CardInt::CardQs,
//!     CardInt::CardJs, CardInt::CardTs,
//! ];
//!
//! assert_eq!(suit_bitwise_and(royal_flush), 0x1);   // spades
//! assert_eq!(rank_bitwise_or(royal_flush), 0x1F00); // A K Q J T bits
//! assert_eq!(prime_product(royal_flush), 41 * 37 * 31 * 29 * 23);
//! ```

use crate::CardInt;

/// Returns the suit nibble common to all cards in the hand, or `0` if the
/// hand is not suited.
///
/// ANDs the suit nibbles (bits 12–15) of every card together, starting from
/// the all-ones mask `0xF000`. Because each suit is a distinct one-hot bit,
/// the result is non-zero only when every card shares the same suit bit.
///
/// # Returns
/// A `u8` with one of the suit bits set (`0x1` spades, `0x2` hearts,
/// `0x4` diamonds, `0x8` clubs) when the hand is flush, or `0x0` otherwise.
pub fn suit_bitwise_and(hand: &[CardInt]) -> u8 {
    (hand.iter().fold(0xF000, |a, b| a & *b as u32) >> 12) as u8
}

/// Returns a bitmask with one bit set per distinct rank present in the hand.
///
/// ORs the one-hot rank bits (bits 16–28) of every card together and shifts
/// the result down to a 13-bit value where bit 0 represents a Deuce and
/// bit 12 represents an Ace.
///
/// # Returns
/// A `u16` rank-presence bitmask. For example, a hand containing `[2s, 3s,
/// 4s, 5s, 6s]` returns `0x001F` and `[As]` returns `0x1000`.
pub fn rank_bitwise_or(hand: &[CardInt]) -> u16 {
    (hand.iter().fold(0, |a, b| a | *b as u32) >> 16) as u16
}

/// Returns the product of the prime numbers assigned to each card's rank.
///
/// Multiplies the prime values stored in bits 0–5 of each card's Cactus Kev
/// encoding. Because each rank maps to a unique prime (deuce = 2, trey = 3,
/// ..., ace = 41), the product uniquely identifies any unordered multiset of
/// ranks and is used for fast hand-rank lookups.
///
/// # Returns
/// A `u32` prime product. Panics on overflow only if the hand is
/// pathologically large; a standard 5-card hand is well within range.
pub fn prime_product(hand: &[CardInt]) -> u32 {
    hand.iter().fold(1, |a, b| a * (*b as u32 & 0xFF))
}

#[cfg(test)]
mod blog_example_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(&[CardInt::CardAs], 0x1)]
    #[case(&[CardInt::CardAs, CardInt::CardKh], 0x0)]
    #[case(&[CardInt::CardAs, CardInt::CardKs, CardInt::CardQs, CardInt::CardJs, CardInt::CardTs], 0x1)]
    #[case(&[CardInt::CardAh, CardInt::CardKh, CardInt::CardQh, CardInt::CardJh, CardInt::CardTh], 0x2)]
    #[case(&[CardInt::CardAd, CardInt::CardKd, CardInt::CardQd, CardInt::CardJd, CardInt::CardTd], 0x4)]
    #[case(&[CardInt::CardAc, CardInt::CardKc, CardInt::CardQc, CardInt::CardJc, CardInt::CardTc], 0x8)]
    #[case(&[CardInt::CardAs, CardInt::CardKh, CardInt::CardQd, CardInt::CardJc, CardInt::CardTs], 0x0)]
    fn test_suit_bitwise_and(#[case] hand: &[CardInt], #[case] expected: u8) {
        assert_eq!(suit_bitwise_and(hand), expected);
    }

    #[rstest]
    #[case(&[CardInt::Card2s], 0x0001)]
    #[case(&[CardInt::CardAs], 0x1000)]
    #[case(&[CardInt::Card2s, CardInt::Card3s, CardInt::Card4s, CardInt::Card5s, CardInt::Card6s], 0x001F)]
    #[case(&[CardInt::CardAs, CardInt::CardKs, CardInt::CardQs, CardInt::CardJs, CardInt::CardTs], 0x1F00)]
    fn test_rank_bitwise_or(#[case] hand: &[CardInt], #[case] expected: u16) {
        assert_eq!(rank_bitwise_or(hand), expected);
    }

    #[rstest]
    #[case(&[CardInt::Card2s], 2)]
    #[case(&[CardInt::CardAs], 41)]
    #[case(&[CardInt::Card2s, CardInt::Card2h, CardInt::Card2d, CardInt::Card2c, CardInt::Card3s], 48)]
    #[case(&[CardInt::CardAs, CardInt::CardAh, CardInt::CardAd, CardInt::CardAc, CardInt::CardKs], 104_553_157)]
    fn test_prime_product(#[case] hand: &[CardInt], #[case] expected: u32) {
        assert_eq!(prime_product(hand), expected);
    }
}
