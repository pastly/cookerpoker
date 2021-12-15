use super::players::PlayerId;
use super::BetAction;
use derive_more::{Add, AddAssign, Div, From, Mul, Rem, Sub, SubAssign, Sum};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Default,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Div,
    Rem,
    Mul,
    Sum,
    From,
    Serialize,
    Deserialize,
    derive_more::Deref,
)]
pub struct Currency(i32);

impl Currency {
    fn max() -> Self {
        i32::MAX.into()
    }
}

/// Handles all pot related operations.
/// Only tracks monies committed to the pot.
/// As such, does no error handling and cannot fail.
/// Parent must validate player has enough monies, and track the state of the betting round.
#[derive(Debug)]
pub struct Pot {
    players_in: HashMap<PlayerId, Currency>,
    max_in: Currency,
    side_pot: Option<Box<Pot>>,
    is_settled: bool,
}

impl Pot {
    /// Returns the total value in this pot
    /// Not particularily useful due to each betting round spinning off a side pot
    pub fn value(&self) -> Currency {
        self.players_in.values().copied().sum()
    }

    pub fn total_value(&self) -> Currency {
        let mut v = self.players_in.values().copied().sum();
        if let Some(x) = self.side_pot.as_ref() {
            v += x.total_value();
        }
        v
    }

    fn overflowing_add(&mut self, player: PlayerId, amount: Currency) {
        if self.is_settled {
            self.side_pot().overflowing_add(player, amount);
        } else {
            let ov = self.players_in.get(&player).copied().unwrap_or_default();
            let nv = ov + amount;
            if nv > self.max_in {
                self.players_in.insert(player, self.max_in);
                let o = nv - self.max_in;
                self.side_pot().overflowing_add(player, o);
            } else {
                self.players_in.insert(player, nv);
            }
        }
    }

    fn side_pot(&mut self) -> &mut Pot {
        if self.side_pot.is_none() {
            self.side_pot = Some(Box::new(Pot::default()));
        }
        self.side_pot.as_mut().unwrap()
    }

    fn update_max(&mut self, new_max: Currency) {
        use std::cmp::Ordering;
        if self.is_settled {
            self.side_pot().update_max(new_max);
        } else {
            if new_max == Currency::max() || new_max < 1.into() {
                return;
            }
            match new_max.cmp(&self.max_in) {
                Ordering::Greater => self.side_pot().update_max(new_max),
                Ordering::Less => {
                    let ov = self.max_in;
                    self.max_in = new_max;
                    if ov != Currency::max() {
                        self.side_pot().update_max(ov - new_max);
                    }
                }
                Ordering::Equal => (),
            }
        }
    }

    /// Parent MUST call this in between betting rounds.
    /// Closes the betting round of all open pots. Next betting roung will create a fresh pot.
    /// This prevents confusion between max_in and next betting rounds.
    pub fn finalize_round(&mut self) {
        self.is_settled = true;
        if let Some(x) = self.side_pot.as_mut() {
            x.finalize_round();
        }
    }

    /// Detected a change in max_bet that could have consquences, forcing a rebuild
    fn overflow(&mut self) {
        if self.is_settled {
            self.side_pot().overflow();
        } else {
            for (player, value) in self.players_in.clone() {
                if value > self.max_in {
                    let delta = value - self.max_in;
                    self.players_in.insert(player, self.max_in);
                    self.overflowing_add(player, delta);
                }
            }
        }
    }

    /// Takes a vector of player Ids and returns the count of them that are in the current pot
    fn num_players_in(&self, hand: &[PlayerId]) -> usize {
        let mut r = 0;
        for i in hand {
            if self.players_in.contains_key(i) {
                r += 1;
            }
        }
        r
    }

    /// Consumes the pot and returns the total payout.
    ///
    /// # Panics
    ///
    /// Panics if the pot would pay out a different amount than is in the pot.
    /// This indicates a failure of the payout function and should be investigated.
    pub fn payout(self, ranked_hands: &[Vec<PlayerId>]) -> HashMap<PlayerId, Currency> {
        let mut hm: HashMap<PlayerId, Currency> = HashMap::new();
        let value = self.value();
        let mut paid_out = false;
        for best_hand in ranked_hands {
            let hands_in = self.num_players_in(best_hand);
            // Prevents divide by zero below
            if hands_in == 0 {
                continue;
            }
            let payout = value / (self.num_players_in(best_hand) as i32);
            for player in best_hand.iter() {
                if self.players_in.contains_key(player) {
                    hm.insert(*player, payout);
                    paid_out = true;
                    if best_hand.len() > 1 && value % 2 == 1.into() {
                        hm.insert(best_hand[0], payout + 1.into());
                    }
                }
            }
            if paid_out {
                break;
            }
        }
        assert_eq!(hm.values().copied().sum::<Currency>(), self.value());
        if let Some(x) = self.side_pot {
            crate::util::merge_hashmap(&mut hm, x.payout(ranked_hands));
        }
        hm
    }

    /// Takes the players TOTAL bet. I.e. Bet(10), Call(20) = bet of 20.
    /// As such, parent must track the current betting round.
    pub fn bet<A: Into<PlayerId>>(&mut self, player: A, action: BetAction) -> Currency {
        use std::cmp::Ordering;
        let player = player.into();
        if self.is_settled {
            self.side_pot().bet(player, action)
        } else {
            let ov = self.players_in.get(&player).copied().unwrap_or_default();
            let value = match action {
                BetAction::AllIn(v) => match v.cmp(&self.max_in) {
                    Ordering::Greater => {
                        let nv = v - self.max_in - ov;
                        self.players_in.insert(player, self.max_in);
                        self.side_pot().bet(player, BetAction::AllIn(nv))
                    }
                    Ordering::Equal => v,
                    Ordering::Less => {
                        self.update_max(v);
                        self.overflow();
                        v
                    }
                },
                BetAction::Bet(v) | BetAction::Call(v) | BetAction::Raise(v) => v,
                // Folds and checks have no effect on the pot.
                BetAction::Fold | BetAction::Check => return 0.into(),
            };
            self.overflowing_add(player, value - ov);
            0.into()
        }
    }
}

impl Default for Pot {
    fn default() -> Self {
        Pot {
            players_in: HashMap::new(),
            max_in: Currency::max(),
            side_pot: None,
            is_settled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Call(5.into()));
        p.bet(3, BetAction::Call(5.into()));
        let payout = p.payout(&vec![vec![1.into()]]);
        assert_eq!(payout[&1.into()], 15.into());
    }

    #[test]
    fn multi_winners() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Bet(5.into()));
        p.bet(3, BetAction::Bet(5.into()));
        let payout = p.payout(&vec![vec![1.into(), 2.into()]]);
        assert_eq!(payout[&1.into()], 8.into());
        assert_eq!(payout[&2.into()], 7.into());

        // it is not possible for the 3rd person to be in for more than the others like this, but
        // the pot does its best to function anyway. Garbage in => garbage out. It's the caller's
        // fault for not knowing how Texas Holdem works.
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Bet(5.into()));
        p.bet(3, BetAction::Bet(6.into()));
        let payout = p.payout(&vec![vec![1.into(), 2.into()]]);
        assert_eq!(payout[&1.into()], 8.into());
        assert_eq!(payout[&2.into()], 8.into());
    }

    #[test]
    fn three_way_tie() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Bet(5.into()));
        p.bet(3, BetAction::Bet(5.into()));
        let payout = p.payout(&vec![vec![1.into(), 2.into(), 3.into()]]);
        dbg!(&payout);
        assert_eq!(payout[&1.into()], 5.into());
        assert_eq!(payout[&2.into()], 5.into());
        assert_eq!(payout[&3.into()], 5.into());
    }

    #[test]
    fn all_in_blind() {
        let mut p = Pot::default();
        p.bet(1, BetAction::AllIn(5.into()));
        p.bet(2, BetAction::Bet(10.into()));
        p.bet(3, BetAction::AllIn(8.into()));
        dbg!(&p);
        let payout = p.payout(&vec![vec![1.into()], vec![2.into(), 3.into()]]);
        dbg!(&payout);
        assert_eq!(payout[&1.into()], 15.into());
        assert_eq!(payout[&2.into()], 5.into());
        assert_eq!(payout[&3.into()], 3.into());
    }

    #[test]
    fn side_pot_payout() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10.into()));
        p.bet(2, BetAction::AllIn(5.into()));
        p.bet(3, BetAction::Bet(10.into()));
        let payout = p.payout(&vec![vec![2.into()], vec![1.into(), 3.into()]]);
        assert_eq!(payout[&2.into()], 15.into());
        assert_eq!(payout[&1.into()], 5.into());
        assert_eq!(payout[&3.into()], 5.into());
    }

    #[test]
    fn overflowing_side_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10.into()));
        p.bet(2, BetAction::AllIn(5.into()));
        p.bet(3, BetAction::AllIn(3.into()));
        dbg!(&p);
        let payout = p.payout(&vec![vec![3.into()], vec![2.into()], vec![1.into()]]);
        dbg!(&payout);
        assert_eq!(payout[&3.into()], 9.into());
        assert_eq!(payout[&2.into()], 4.into());
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1.into()], 5.into());
    }

    #[test]
    fn multi_round_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Call(5.into()));
        p.bet(3, BetAction::Call(5.into()));
        p.finalize_round();
        // 5,5,5 = 15 in pot
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Bet(10.into()));
        p.bet(3, BetAction::AllIn(8.into()));
        p.bet(1, BetAction::Call(10.into()));
        p.finalize_round();
        // 15 + 8,8,8 + 2,2 = 43 in pot
        p.bet(1, BetAction::Bet(10.into()));
        p.bet(2, BetAction::AllIn(6.into()));
        p.finalize_round();
        // 43 + 6,6 + 4 = 59 in pot
        dbg!(&p);
        let payout = p.payout(&vec![vec![3.into()], vec![2.into()], vec![1.into()]]);
        dbg!(&payout);
        assert_eq!(payout[&3.into()], 39.into());
        assert_eq!(payout[&2.into()], 16.into());
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1.into()], 4.into());
    }

    #[test]
    /// bet, call, and raise are all semantically the same as far as the pot is concerned.
    fn bet_call_raise() {
        fn helper(p: Pot) {
            assert_eq!(p.players_in.len(), 3);
            for v in p.players_in.values() {
                assert_eq!(*v, 5.into());
            }
            assert_eq!(p.max_in, Currency::max());
            assert!(p.side_pot.is_none());
            dbg!(&p);
            let payout = p.payout(&vec![vec![1.into()]]);
            assert_eq!(payout[&(1.into())], 15.into());
            dbg!(&payout);
        }
        let mut p1 = Pot::default();
        p1.bet(1, BetAction::Bet(5.into()));
        p1.bet(2, BetAction::Bet(5.into()));
        p1.bet(3, BetAction::Bet(5.into()));
        p1.finalize_round();
        helper(p1);
        let mut p2 = Pot::default();
        p2.bet(1, BetAction::Call(5.into()));
        p2.bet(2, BetAction::Call(5.into()));
        p2.bet(3, BetAction::Call(5.into()));
        p2.finalize_round();
        helper(p2);
        let mut p3 = Pot::default();
        p3.bet(1, BetAction::Raise(5.into()));
        p3.bet(2, BetAction::Raise(5.into()));
        p3.bet(3, BetAction::Raise(5.into()));
        p3.finalize_round();
        helper(p3);
    }

    #[test]
    fn multi_round_pot2() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Call(5.into()));
        p.bet(3, BetAction::Raise(15.into()));
        p.bet(1, BetAction::Call(15.into()));
        p.bet(2, BetAction::Call(15.into()));
        p.finalize_round();
        assert_eq!(p.total_value(), 45.into());
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::AllIn(50.into()));
        p.bet(3, BetAction::Call(50.into()));
        p.bet(1, BetAction::Raise(500.into()));
        // 2 is all in and can't do anything
        // 3 folds, so there's nothing more to do
        p.finalize_round();
        // lets pretend that's the end and make sure the pots are exactly as expected
        dbg!(&p);

        assert_eq!(p.players_in.len(), 3);
        for v in p.players_in.values() {
            assert_eq!(*v, 15.into());
        }
        assert_eq!(p.max_in, Currency::max());

        let side_pot = p.side_pot.unwrap();
        assert_eq!(side_pot.players_in.len(), 3);
        for v in side_pot.players_in.values() {
            assert_eq!(*v, 50.into());
        }
        assert_eq!(side_pot.max_in, 50.into());

        let side_pot = side_pot.side_pot.unwrap();
        assert_eq!(side_pot.players_in.len(), 1);
        for v in side_pot.players_in.values() {
            assert_eq!(*v, 450.into());
        }
        assert_eq!(side_pot.max_in, Currency::max());
    }
}
