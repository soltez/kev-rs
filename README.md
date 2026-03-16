# kev-rs

A Rust implementation of Cactus Kev's 32-bit card integer encoding, with hand-evaluation primitives for poker hand analysis.

## Bit layout

Each playing card is packed into a single `u32`:

```text
+--------+--------+--------+--------+
|xxxbbbbb|bbbbbbbb|cdhsrrrr|xxpppppp|
+--------+--------+--------+--------+

Bits 28–16: b = one-hot rank bit
Bits 15–12: cdhs = one-hot suit nibble (c=clubs, d=diamonds, h=hearts, s=spades)
Bits 11– 8: r = rank index (deuce=0, trey=1, ..., ace=12)
Bits  5– 0: p = rank prime  (deuce=2, trey=3, ..., ace=41)
```

## Usage

### Cards

Cards can be accessed as enum variants or constructed from a two-character string:

```rust
use kev::CardInt;

let ace_of_spades = CardInt::CardAs;
let king_of_clubs = CardInt::new("Kc").unwrap();

assert_eq!(ace_of_spades.rank(), king_of_clubs.rank()); // false — different ranks
assert_eq!(CardInt::new("Ac").unwrap().suit(), king_of_clubs.suit()); // same suit
```

Variants are named `Card<Rank><Suit>` where rank uses its conventional character
(`A K Q J T 9 8 7 6 5 4 3 2`) and suit uses its initial (`s h d c`).

`CardInt::new` accepts any two-character string and returns a `CardError` for
invalid rank, invalid suit, or wrong length.

### Hand evaluation primitives

The `hand` module exposes three functions over a slice of `CardInt` values:

| Function | Bits used | Purpose |
|---|---|---|
| `suit_bitwise_and` | 12–15 (suit nibble) | Flush detection |
| `rank_bitwise_or`  | 16–28 (rank one-hot) | Straight / rank-presence mask |
| `prime_product`    | 0–5 (prime byte)     | Unique rank-multiset key |

```rust
use kev::CardInt;
use kev::hand::{suit_bitwise_and, rank_bitwise_or, prime_product};

let royal_flush = &[
    CardInt::CardAs, CardInt::CardKs, CardInt::CardQs,
    CardInt::CardJs, CardInt::CardTs,
];

assert_eq!(suit_bitwise_and(royal_flush), 0x1);            // spades
assert_eq!(rank_bitwise_or(royal_flush), 0x1F00);          // A K Q J T bits set
assert_eq!(prime_product(royal_flush), 41 * 37 * 31 * 29 * 23);
```

- **`suit_bitwise_and`** — returns the common suit nibble (`0x1`/`0x2`/`0x4`/`0x8`) if all cards share a suit, or `0x0` otherwise.
- **`rank_bitwise_or`** — returns a 13-bit mask with one bit set per distinct rank; useful for straight detection.
- **`prime_product`** — returns the product of each rank's unique prime, uniquely identifying any unordered multiset of ranks for fast lookup-table indexing.
