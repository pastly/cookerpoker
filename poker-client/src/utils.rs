use poker_core::cards::{card::Rank, card::Suit, Card};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub(crate) fn card_char(card: Card) -> char {
    // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode#Block
    let base: u32 = match card.suit {
        Suit::Spade => 0x1F0A0,
        Suit::Heart => 0x1F0B0,
        Suit::Diamond => 0x1F0C0,
        Suit::Club => 0x1F0D0,
    };
    let val = base
        + match card.rank {
            Rank::Ace => 1,
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            // Unicode includes Knight here. Skip 12.
            Rank::Queen => 13,
            Rank::King => 14,
        };
    // Safety: Value will always be a valid char thanks to match statements and enums on card
    // suits and ranks.
    unsafe { std::char::from_u32_unchecked(val) }
}

pub(crate) fn _char_card(c: char) -> Option<Card> {
    let c = c as u32;
    let (base, suit): (u32, _) = {
        if c > 0x1F0D0 {
            (0x1F0D0, Suit::Club)
        } else if c > 0x1F0C0 {
            (0x1F0C0, Suit::Diamond)
        } else if c > 0x1F0B0 {
            (0x1F0B0, Suit::Heart)
        } else if c > 0x1F0A0 {
            (0x1F0A0, Suit::Spade)
        } else {
            return None;
        }
    };
    let rank = {
        let diff = c - base;
        match diff {
            1 => Rank::Ace,
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Jack,
            // Unicode includes Knight here. Skip 12.
            13 => Rank::Queen,
            14 => Rank::King,
            _ => return None,
        }
    };
    Some(Card::new(suit, rank))
}
