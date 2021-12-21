use super::players::PlayerId;
use super::BetAction;
use derive_more::{Add, AddAssign, Div, From, Mul, Rem, Sub, SubAssign, Sum};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::os::unix::prelude::OsStringExt;

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

/// Players put Stakes in Pots. Binds an is_allin flag to the bet amount, as an important part of
/// pot logic is keeping track of AllIn-related limits on winnings.
#[derive(Debug, Copy, Clone)]
struct Stake {
    is_allin: bool,
    amount: Currency,
}

impl From<(bool, Currency)> for Stake {
    fn from(tup: (bool, Currency)) -> Self {
        Self {
            is_allin: tup.0,
            amount: tup.1,
        }
    }
}

/// Divide X as evenly as possible Y ways using only positive ints, and return those ints.
///
/// Consider x=5 and y=3. 5 cannot be divided into 3 pieces evenly using ints. This function would
/// return vec![2, 2, 1].
/// x=8, y=5 returns 2, 2, 2, 1, 1.
/// x=6, y=3 returns 2, 2, 2
///
/// # Panics
///
/// Panics if provided negative numbers. There should never be a negative payout, or a negative number of players
fn split_x_by_y(x: i32, y: i32) -> Vec<i32> {
    assert!(y.is_positive());
    assert!(x.is_positive());
    let mut ret = Vec::with_capacity(y as usize);
    let mut frac_accum = 0;
    for i in 0..y {
        frac_accum += x % y;
        if frac_accum >= y || i == y - 1 && frac_accum > 0 {
            ret.push((x / y) + 1);
        } else {
            ret.push(x / y);
        }
        if frac_accum >= y {
            frac_accum -= y;
        }
    }
    ret.sort_unstable();
    ret.reverse();
    ret
}

/// Handles all pot related operations.
/// Only tracks monies committed to the pot.
/// As such, does no error handling and cannot fail.
/// Parent must validate player has enough monies, and track the state of the betting round.
#[derive(Debug)]
pub struct Pot {
    settled: Vec<InnerPot>,
    working: InnerPot,
}

#[derive(Debug)]
struct InnerPot {
    players_in: HashMap<PlayerId, Stake>,
    max_in: Option<Currency>,
    inner: Option<Box<InnerPot>>,
}

impl InnerPot {
    fn inner(&mut self) -> &mut Self {
        if self.inner.is_none() {
            self.inner = Some(Box::new(Self::default()));
        }
        self.inner.as_mut().unwrap()
    }

    /// Consume ourselves and all our nested InnerPots, returning us all as a vector. We are first,
    /// and the depeest InnerPot below us in the chain is last.
    fn vectorize(mut self) -> Vec<Self> {
        // remove the InnerPot from this InnerPot, if any
        let inner = self.inner.take();
        // init our return Vec with ourselves, so that we are farthest to the left and the deepest
        // InnerPot will be farthest to the right.
        let mut v = vec![self];
        // if there's a nested inner pot, go recursive and append the results to the return Vec.
        if let Some(inner) = inner {
            v.append(&mut (*inner).vectorize());
        }
        v
    }

    /// Recursive helper for sort. Pops off a pot from the right of pots, makes it the inner pot
    /// of the given self pot, and goes recursive from there.
    ///
    /// Pots farther to the right end up being higher in the chain.
    fn restack(mut self, mut pots: Vec<Self>) -> Box<Self> {
        if pots.is_empty() {
            self.inner = None;
            return Box::new(self);
        }
        let inner = pots.pop().unwrap();
        self.inner = Some(inner.restack(pots));
        Box::new(self)
    }

    /// Consume ourself and resort ourself and our inner pots such that the inner pot with the
    /// lowest max_in is nearest to the top.
    ///
    /// Pots with no max_in are considered equal.
    fn sort(self) -> Self {
        let mut pots = self.vectorize();
        // Sort the pots such that:
        // - Nones are farthest to the left in arbitrary order,
        // - Somes are to the right of Nones,
        // - Somes are ordered by their max_in such that the largest max_in is to the left of smaller ones.
        pots.sort_unstable_by(|l, r| match (l.max_in, r.max_in) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(lv), Some(rv)) => rv.cmp(&lv),
        });
        let top = pots.pop().unwrap();
        *top.restack(pots)
    }

    /// Helper for [`Pot::bet`] that automatically handles the creation of side pots as needed.
    fn bet_helper(&mut self, player: PlayerId, stake: Stake) {
        match (stake.is_allin, self.max_in) {
            (false, None) => {
                self.players_in.insert(player, stake);
            }
            (true, None) => {
                self.players_in.insert(player, stake);
                let max_in = stake.amount;
                self.max_in = Some(stake.amount);
                // there may be bets in the pot that are larger than this AllIn. Need to overflow
                let buf: Vec<(PlayerId, Stake)> = self.players_in.drain().collect();
                for (p, s) in buf.into_iter() {
                    match s.amount.cmp(&max_in) {
                        Ordering::Less | Ordering::Equal => {
                            self.players_in.insert(p, s);
                        }
                        Ordering::Greater => {
                            self.players_in.insert(p, (s.is_allin, max_in).into());
                            self.inner()
                                .bet_helper(p, (s.is_allin, s.amount - max_in).into());
                        }
                    }
                }
            }
            (false, Some(max_in)) => {
                // Put as much of this bet as possible in this pot, and put the rest in additional
                // pots. This new bet isn't an AllIn, so no overflowing is necessary. Just put as
                // much as possible in this pot, and the remainder in the next (by calling this
                // function, and maybe having to do as much as possible again with remainder in the
                // next, but that's ok.)
                match stake.amount.cmp(&max_in) {
                    Ordering::Less | Ordering::Equal => {
                        self.players_in.insert(player, stake);
                    }
                    Ordering::Greater => {
                        self.players_in.insert(player, (false, max_in).into());
                        self.inner()
                            .bet_helper(player, (false, stake.amount - max_in).into());
                    }
                }
            }
            (true, Some(max_in)) => {
                // Like the previous match arm, we want to put as much as possible in each pot
                // before moving on to the next. But this is complicated by the following:
                // - This AllIn could be less or greater than the existing max.
                //     - If less, the existing max should be lowered and any overflow from existing
                //       bets moved for each player into the next pot.
                //     - If greater, as much as possible is put in this pot, and the remainder in
                //       the next.
                match stake.amount.cmp(&max_in) {
                    Ordering::Equal => {
                        self.players_in.insert(player, stake);
                    }
                    Ordering::Greater => {
                        self.players_in.insert(player, (true, max_in).into());
                        self.inner()
                            .bet_helper(player, (true, stake.amount - max_in).into());
                    }
                    Ordering::Less => {
                        self.players_in.insert(player, stake);
                        // update this pot's max to this new lower amount
                        let max_in = stake.amount;
                        self.max_in = Some(max_in);
                        // Collect all players (incl. this new one) in a buffer. Many may go right
                        // back into this pot unchanged, but some need their amount lowered and the
                        // remainder sent into the next pot. This was the most logical way I came
                        // up with for doing this.
                        let buf: Vec<(PlayerId, Stake)> = self.players_in.drain().collect();
                        for (p, s) in buf.into_iter() {
                            match s.amount.cmp(&max_in) {
                                // If the player is is for less or equal to the new max amount,
                                // just put them right back in.
                                Ordering::Less | Ordering::Equal => {
                                    self.players_in.insert(p, s);
                                }
                                // Otherwise, put them in for the new (lower) max amount, and send
                                // the overflow into the next pot.
                                Ordering::Greater => {
                                    self.players_in.insert(p, (s.is_allin, max_in).into());
                                    self.inner()
                                        .bet_helper(p, (s.is_allin, s.amount - max_in).into());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn payout(self, ranked_players: &[Vec<PlayerId>]) -> HashMap<PlayerId, Currency> {
        let mut hm: HashMap<PlayerId, Currency> = HashMap::new();
        for player_group in ranked_players {
            let players = {
                let mut v = vec![];
                for p in player_group {
                    if self.players_in.contains_key(p) {
                        v.push(*p);
                    }
                }
                v
            };
            // Prevents divide by zero below.
            if players.is_empty() {
                continue;
            }
            let payouts = split_x_by_y(*self.value(), players.len().try_into().unwrap());
            assert_eq!(players.len(), payouts.len());
            for (player, payout) in itertools::zip(players, payouts) {
                hm.insert(player, payout.into());
            }
            break;
        }
        hm
    }

    fn value(&self) -> Currency {
        self.players_in.values().copied().map(|s| s.amount).sum()
    }
}
impl Pot {
    /// Call this function between rounds to mark the betting round as over and all working pots
    /// settled.
    /// 
    /// InnerPots are stored such that the oldest is farthest to the left of the settled vector.
    /// There are fewer and fewer players in the game as you move to the right of the settled vec.
    /// If a game goes all the way to showdown, the right-most InnerPot of settled is the final pot
    /// with the final players in it.
    pub(crate) fn finalize_round(&mut self) {
        let working = std::mem::replace(&mut self.working, InnerPot::default());
        let mut v = working.vectorize();
        while let Some(p) = v.pop() {
            self.settled.push(p);
        }
    }

    #[cfg(test)]
    fn settled_value(&self) -> Currency {
        let mut ret = 0.into();
        for p in &self.settled {
            ret += p.value();
        }
        ret
    }

    /// Consumes the pot and returns the total payout.
    ///
    /// # Panics
    ///
    /// Panics if the pot would pay out a different amount than is in the pot.
    /// This indicates a failure of the payout function and should be investigated.
    pub(crate) fn payout(self, ranked_players: &[Vec<PlayerId>]) -> HashMap<PlayerId, Currency> {
        let mut hm: HashMap<PlayerId, Currency> = HashMap::new();
        for pot in self.settled {
            crate::util::merge_hashmap(&mut hm, pot.payout(&ranked_players));
        }
        hm
    }

    /// Takes the players TOTAL bet. I.e. Bet(10), Call(20) = bet of 20.
    /// As such, parent must track the current betting round.
    pub(crate) fn bet<A: Into<PlayerId>>(&mut self, player: A, action: BetAction) {
        let player = player.into();
        // handle the bet
        let stake = match action {
            BetAction::Check | BetAction::Fold => {
                return;
            }
            BetAction::Call(v) | BetAction::Bet(v) | BetAction::Raise(v) => (false, v),
            BetAction::AllIn(v) => (true, v),
        }
        .into();
        self.working.bet_helper(player, stake);
        // pull out the working InnerPot(s) so that they can be sorted, then put them back under self.
        let working = std::mem::replace(&mut self.working, InnerPot::default());
        self.working = working.sort();
    }
}

impl Default for Pot {
    fn default() -> Self {
        Pot {
            settled: vec![],
            working: InnerPot::default(),
        }
    }
}

impl Default for InnerPot {
    fn default() -> Self {
        Self {
            players_in: HashMap::new(),
            max_in: None,
            inner: None,
        }
    }
}

#[cfg(test)]
mod test_innerpot_sorting {
    use super::*;

    fn ip(max_in: Option<Currency>) -> InnerPot {
        let mut ip = InnerPot::default();
        ip.max_in = max_in;
        ip
    }

    fn one_to_five_then_none(ip: InnerPot) {
        dbg!(&ip);
        let maxes: Vec<Option<i32>> = ip
            .vectorize()
            .into_iter()
            .map(|p| match p.max_in {
                None => None,
                Some(v) => Some(*v),
            })
            .collect();
        assert_eq!(maxes, vec![Some(1), Some(2), Some(3), Some(4), None]);
    }

    #[test]
    fn in_order() {
        let mut ip1 = ip(Some(1.into()));
        let mut ip2 = ip(Some(2.into()));
        let mut ip3 = ip(Some(3.into()));
        let mut ip4 = ip(Some(4.into()));
        let ip5 = ip(None);

        ip4.inner = Some(Box::new(ip5));
        ip3.inner = Some(Box::new(ip4));
        ip2.inner = Some(Box::new(ip3));
        ip1.inner = Some(Box::new(ip2));

        let ip = ip1.sort();
        one_to_five_then_none(ip);
    }

    #[test]
    fn in_order_rev() {
        let ip1 = ip(Some(1.into()));
        let mut ip2 = ip(Some(2.into()));
        let mut ip3 = ip(Some(3.into()));
        let mut ip4 = ip(Some(4.into()));
        let mut ip5 = ip(None);

        ip2.inner = Some(Box::new(ip1));
        ip3.inner = Some(Box::new(ip2));
        ip4.inner = Some(Box::new(ip3));
        ip5.inner = Some(Box::new(ip4));

        let ip = ip5.sort();
        one_to_five_then_none(ip);
    }

    #[test]
    fn random1() {
        let mut ip1 = ip(Some(1.into()));
        let mut ip2 = ip(Some(2.into()));
        let ip3 = ip(Some(3.into()));
        let mut ip4 = ip(Some(4.into()));
        let mut ip5 = ip(None);

        ip1.inner = Some(Box::new(ip3));
        ip4.inner = Some(Box::new(ip1));
        ip2.inner = Some(Box::new(ip4));
        ip5.inner = Some(Box::new(ip2));

        let ip = ip5.sort();
        one_to_five_then_none(ip);
    }

    fn one_then_nones(ip: InnerPot, n_nones: usize) {
        dbg!(&ip);
        let maxes: Vec<Option<i32>> = ip
            .vectorize()
            .into_iter()
            .map(|p| match p.max_in {
                None => None,
                Some(v) => Some(*v),
            })
            .collect();
        let mut expect = vec![Some(1)];
        expect.extend((0..n_nones).map(|_| None));
        assert_eq!(maxes, expect);
    }

    #[test]
    fn one_nonnone1() {
        let mut ip1 = ip(Some(1.into()));
        let ip2 = ip(None);

        ip1.inner = Some(Box::new(ip2));

        let ip = ip1.sort();
        one_then_nones(ip, 1);
    }

    #[test]
    fn one_nonnone2() {
        let ip1 = ip(Some(1.into()));
        let mut ip2 = ip(None);

        ip2.inner = Some(Box::new(ip1));

        let ip = ip2.sort();
        one_then_nones(ip, 1);
    }

    #[test]
    fn one_nonnone3() {
        let mut ip1 = ip(None);
        let mut ip2 = ip(Some(1.into()));
        let ip3 = ip(None);

        ip2.inner = Some(Box::new(ip3));
        ip1.inner = Some(Box::new(ip2));

        let ip = ip1.sort();
        one_then_nones(ip, 2);
    }

    #[test]
    fn one_nonnone4() {
        let mut ip1 = ip(Some(1.into()));
        let mut ip2 = ip(None);
        let ip3 = ip(None);

        ip2.inner = Some(Box::new(ip3));
        ip1.inner = Some(Box::new(ip2));

        let ip = ip1.sort();
        one_then_nones(ip, 2);
    }

    #[test]
    fn one_nonnone5() {
        let mut ip1 = ip(None);
        let mut ip2 = ip(None);
        let ip3 = ip(Some(1.into()));

        ip2.inner = Some(Box::new(ip3));
        ip1.inner = Some(Box::new(ip2));

        let ip = ip1.sort();
        one_then_nones(ip, 2);
    }
}

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
            let ip = &p.settled[0];
            assert_eq!(ip.players_in.len(), 3);
            for v in ip.players_in.values() {
                assert_eq!(v.amount, 5.into());
            }
            assert_eq!(ip.max_in, None);
            assert!(ip.inner.is_none());
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
        assert_eq!(p.settled_value(), 45.into());
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::AllIn(50.into()));
        p.bet(3, BetAction::Call(50.into()));
        p.bet(1, BetAction::Raise(500.into()));
        // 2 is all in and can't do anything
        // 3 folds, so there's nothing more to do
        p.finalize_round();
        // lets pretend that's the end and make sure the pots are exactly as expected
        dbg!(&p);

        let pot = &p.settled[0];
        assert_eq!(pot.players_in.len(), 3);
        for v in pot.players_in.values() {
            assert_eq!(v.amount, 15.into());
        }
        assert_eq!(pot.max_in, None);

        let pot = &p.settled[1];
        assert_eq!(pot.players_in.len(), 3);
        for v in pot.players_in.values() {
            assert_eq!(v.amount, 50.into());
        }
        assert_eq!(pot.max_in, Some(50.into()));

        //let side_pot = side_pot.side_pot.unwrap();
        //assert_eq!(side_pot.players_in.len(), 1);
        //for v in side_pot.players_in.values() {
        //    assert_eq!(*v, 450.into());
        //}
        //assert_eq!(side_pot.max_in, Currency::max());
    }

    // #[test]
    // fn all_all_in() {
    //     let mut p = Pot::default();
    //     p.bet(1, BetAction::AllIn(5.into()));
    //     p.bet(2, BetAction::AllIn(15.into()));
    //     p.bet(3, BetAction::AllIn(45.into()));
    //     p.finalize_round();
    //     //dbg!(&p);
    //     assert_eq!(p.players_in.len(), 3);
    //     assert_eq!(*p.max_in, 5);
    //     let side_pot = p.side_pot.unwrap();
    //     assert_eq!(side_pot.players_in.len(), 2);
    //     assert_eq!(*side_pot.max_in, 10);
    //     let side_pot = side_pot.side_pot.unwrap();
    //     assert_eq!(side_pot.players_in.len(), 1);
    //     assert_eq!(*side_pot.max_in, 30);

    //     let mut p = Pot::default();
    //     p.bet(1, BetAction::AllIn(45.into()));
    //     p.bet(2, BetAction::AllIn(15.into()));
    //     p.bet(3, BetAction::AllIn(5.into()));
    //     p.finalize_round();
    //     dbg!(&p);
    //     assert_eq!(p.players_in.len(), 3);
    //     assert_eq!(*p.max_in, 5);
    //     let side_pot = p.side_pot.unwrap();
    //     assert_eq!(side_pot.players_in.len(), 2);
    //     assert_eq!(*side_pot.max_in, 10);
    //     let side_pot = side_pot.side_pot.unwrap();
    //     assert_eq!(side_pot.players_in.len(), 1);
    //     assert_eq!(*side_pot.max_in, 20);
    //     let side_pot = side_pot.side_pot.unwrap();
    //     assert_eq!(side_pot.players_in.len(), 1);
    //     assert_eq!(*side_pot.max_in, 10);
    // }
}

#[cfg(test)]
mod test_split_x_by_y {
    use super::split_x_by_y;

    #[test]
    fn test1() {
        assert_eq!(split_x_by_y(5, 3), vec![2, 2, 1]);
    }

    #[test]
    fn test2() {
        assert_eq!(split_x_by_y(6, 2), vec![3, 3]);
    }

    #[test]
    fn test3() {
        assert_eq!(split_x_by_y(8, 5), vec![2, 2, 2, 1, 1]);
    }
}
