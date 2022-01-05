use super::players::PlayerId;
use super::BetAction;
use derive_more::{Add, AddAssign, Div, From, Mul, Rem, Sub, SubAssign, Sum};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
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

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let dollars = self.0 / 100;
        let cents = self.0 - (dollars * 100);
        write!(f, "{}.{:02}", dollars, cents)
    }
}

#[derive(Debug)]
pub(crate) enum LogItem {
    Bet(PlayerId, BetAction),
    RoundEnd(usize),
    BetsSorted(Vec<(PlayerId, Stake)>),
    EntireStakeInPot(usize, PlayerId, Stake),
    PartialStakeInPot(usize, PlayerId, Stake, Currency),
    NewPotCreated(usize, PlayerId, Stake),
    Payouts(Option<usize>, HashMap<PlayerId, Currency>),
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LogItem::Bet(player, bet) => write!(f, "Player {} makes bet {}", player, bet),
            LogItem::RoundEnd(settled_n) => write!(
                f,
                "The betting round has ended. There are {} settled pots",
                settled_n
            ),
            LogItem::BetsSorted(bets) => {
                let middle: String = bets
                    .iter()
                    .map(|(player, stake)| format!("p{}: {}", player, stake))
                    .join(", ");
                let s = "[".to_string() + &middle + "]";
                write!(f, "Betting round is ending. Bets are sorted: {}", s)
            }
            LogItem::EntireStakeInPot(pot_n, player, stake) => write!(
                f,
                "Player {}'s bet {} entirely allocated to pot {}",
                player, stake, pot_n
            ),
            LogItem::PartialStakeInPot(pot_n, player, stake, max_in) => write!(
                f,
                "{} of Player {}'s bet {} allocated to pot {}",
                max_in, player, stake, pot_n
            ),
            LogItem::NewPotCreated(pot_n, player, stake) => write!(
                f,
                "Player {}'s bet {} allocated to new pot {}",
                player, stake, pot_n
            ),
            LogItem::Payouts(pot_n, payouts) => {
                let middle: String = payouts
                    .iter()
                    .map(|(player, amount)| format!("p{}: {}", player, amount))
                    .join(", ");
                let s = "[".to_string() + &middle + "]";
                let prefix = match pot_n {
                    None => "Total".to_string(),
                    Some(pot_n) => format!("Settled pot {}", pot_n),
                };
                write!(f, "{} payouts: {}", prefix, s)
            }
        }
    }
}

/// Players put Stakes in Pots. Binds an is_allin flag to the bet amount, as an important part of
/// pot logic is keeping track of AllIn-related limits on winnings.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Stake {
    is_allin: bool,
    amount: Currency,
}

impl std::fmt::Display for Stake {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "({}{})",
            self.amount,
            if self.is_allin { " allin" } else { "" }
        )
    }
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

/// "Public" interface to a pot. Tell the pot when players bet, when betting rounds are over, and
/// the order of winning hands when it's time to pay out, and Pot will take care of all the dirty
/// details.
///
/// Pot only tracks monitary commitments to the pot. It doesn't do error handling and will
/// generally never fail. Pot will not check that the bets make sense. If you put garbage in, you
/// will get garbage out.
///
/// Pot ignores folds. When calling the payout function, only provide players that are still in the
/// hand.
///
/// # Usage
///
/// ## `bet(...)`
///
/// Call this to record every player's bet as they make them. Always use their total commitment in
/// the current betting round. For example, if a player bets 10, gets raised to 30, then calls the
/// raise for a total of 30, you are to give Bet(10), Raise(30), Call(30) (with the appropriate
/// players, of course) as these represent each player's *total* commitment.
///
/// ## `finalize_round()`
///
/// Call this between betting rounds so the working pot can be settled and future betting can
/// establish a new pot. This *must* be called before `payout(...)`.
///
/// ## `payout(...)`
///
/// Immediately after calling `finalize_round()` for the last time, call this to calculate and
/// return the payouts for each winning player. This consumes the Pot. See the function
/// documentation for more information.
#[derive(Debug)]
pub struct Pot {
    /// Pots from previous betting rounds. These will not be changed, and when it comes time to pay
    /// the winner, these are where the funds come from.
    settled: Vec<InnerPot>,
    /// Cache for storing the players that are participating in the current betting round and their
    /// bets. When a betting round is finalized, this is emptied, and InnerPot(s) are created and
    /// added to settled.
    working: HashMap<PlayerId, Stake>,
    /// List of actions we are told about and actions we take, in order. The purpose is to aide in
    /// debugging or explaining why payouts are what they are.
    log: Vec<LogItem>,
}

/// An innner subpot that Pot uses to keep track of pools of money that players can win. New
/// InnerPots are created every betting round and extra ones are created when players go all in.
#[derive(Debug)]
struct InnerPot {
    /// The players that are eligible to win this pot and the stake they put into this pot
    /// (is_allin, amount).
    players: HashMap<PlayerId, Stake>,
    /// The maximum amount a player entering this pot is allowed to bet. This is only non-None for
    /// pots containing 1+ player that is all in.
    max_in: Option<Currency>,
}

impl InnerPot {
    /// For this InnerPot only, return the player(s) that won and the amount they won.
    ///
    /// See Pot's payout function for more information on the ranked_players argument.
    fn payout(self, ranked_players: &[Vec<PlayerId>]) -> HashMap<PlayerId, Currency> {
        let mut hm: HashMap<PlayerId, Currency> = HashMap::new();
        // Loop over the player rank groups. The first group that contains >0 players in this pot is
        // used, and then we are done. So we generally expect to only loop once. Remember, the
        // player(s) with the best hand are in the first group.
        for player_group in ranked_players {
            // See if any of the players in this group were eligible to win this pot. If not, move
            // on to the next group
            let winning_players: Vec<_> = player_group
                .iter()
                .filter(|&&p| self.players.contains_key(&p))
                .collect();
            if winning_players.is_empty() {
                continue;
            }
            assert!(!winning_players.is_empty());
            // split the payout evenly across all the winning players. It's important that we
            // avoided division by 0 by making sure there is >0 winning players.
            let payouts = split_x_by_y(*self.value(), winning_players.len().try_into().unwrap());
            for (player, payout) in itertools::zip(winning_players, payouts) {
                hm.insert(*player, payout.into());
            }
            break;
        }
        hm
    }

    /// Returns the sum of all the bets all players in this pot have made.
    fn value(&self) -> Currency {
        self.players.values().copied().map(|s| s.amount).sum()
    }
}

impl Pot {
    /// Call this function between rounds to mark the betting round as over.
    ///
    /// The cache of players and their bets is turned into one or more InnerPots, which are then
    /// stored in our settled vec of InnerPots (at which point they won't be touched). Side pots are
    /// automatically created if 1+ players have gone all in.
    pub(crate) fn finalize_round(&mut self) {
        // The new pot(s) we will add to our vec of settled pots
        let mut pots: Vec<InnerPot> = vec![];
        // Sort the players that are in this betting round such that:
        // - players that went all in are first, and
        // - a player that are all in for less than another all in player comes first.
        let iter: Vec<_> = self
            .working
            .drain()
            .sorted_unstable_by(|l, r| {
                match (l.1.is_allin, r.1.is_allin) {
                    // both all in, and smallest amount should be first.
                    (true, true) => l.1.amount.cmp(&r.1.amount),
                    // left all in, so must be less than (before) right
                    (true, false) => Ordering::Less,
                    // right all in, so must be less than left
                    (false, true) => Ordering::Greater,
                    // neither all in, so equal
                    (false, false) => Ordering::Equal,
                }
            })
            .collect();
        self.log.push(LogItem::BetsSorted(iter.clone()));
        // Back to actual work. For each player, add their stake to the pots, possibly splitting it
        // up across pots if necessary due to other players going all in.
        for (player, mut stake) in iter {
            for (pot_n, pot) in pots.iter_mut().enumerate() {
                match pot.max_in {
                    // This pot doesn't have an existing all in player, so this player can add an
                    // unlimted amount to it. (In practice, assuming no all ins, the caller should
                    // be verifying that all players are in for the same amount, so "unlimted"
                    // really means "the same exact amount as everyone else").
                    None => {
                        self.log
                            .push(LogItem::EntireStakeInPot(pot_n, player, stake));
                        pot.players.insert(player, stake);
                        // Reduce the amount to 0, indicating to future code that the player's bet
                        // is fully accounted for.
                        stake.amount = 0.into();
                        // and since there is no more amount to add to inner pots, stop iterating
                        // over the inner pots.
                        break;
                    }
                    // THere is an all in player in this pot, so there is a limit on how much this
                    // player can add to it.
                    Some(max_in) => match stake.amount.cmp(&max_in) {
                        // If this player is adding less or equal than the existing limit, then
                        // simply add them to the pot and be done. (In practice, we expect them to
                        // be adding equal as they have called an all in).
                        Ordering::Less | Ordering::Equal => {
                            self.log
                                .push(LogItem::EntireStakeInPot(pot_n, player, stake));
                            pot.players.insert(player, stake);
                            // Indicate the bet is fully accounted for.
                            stake.amount = 0.into();
                            // Stop interating over the pots since no more amount to add to pots.
                            break;
                        }
                        // The player wants to add more than the limit, so add the limit for them
                        // and reduce their stake that shall be put into the next pot(s)
                        Ordering::Greater => {
                            self.log
                                .push(LogItem::PartialStakeInPot(pot_n, player, stake, max_in));
                            pot.players.insert(player, (stake.is_allin, max_in).into());
                            stake.amount -= max_in;
                        }
                    },
                }
            }
            // The player's bet has not been fully accounted for in the existing inner pots. An
            // example of when this would happen is: this is the second player to go all in this
            // betting round, and they've done so for more than the first player. We create a new
            // inner pot for them and add it to the list of pots. Future iterations of this loop
            // with the next players and their bets will add to this pot.
            if stake.amount > 0.into() {
                let mut new = InnerPot {
                    max_in: match stake.is_allin {
                        false => None,
                        true => Some(stake.amount),
                    },
                    ..Default::default()
                };
                new.players.insert(player, stake);
                pots.push(new);
                self.log
                    .push(LogItem::NewPotCreated(pots.len() - 1, player, stake));
            }
        }
        // Finally done creating all the new pots, so move them to settled.
        self.settled.append(&mut pots);
        self.log.push(LogItem::RoundEnd(self.settled.len()));
    }

    /// The value of all InnerPots that are settled and will not change. I.e. funds from previous
    /// betting rounds
    fn settled_value(&self) -> Currency {
        let mut ret = 0.into();
        for sp in &self.settled {
            ret += sp.players.values().copied().map(|s| s.amount).sum();
        }
        ret
    }

    /// The value of all settled and unsettled bets in the pot.
    ///
    /// Settled means funds that are in InnerPots that will not change because they are from
    /// previous betting rounds. Unsettled means they are still potentially going to change due to
    /// calling raises, etc.
    pub(crate) fn total_value(&self) -> Currency {
        self.settled_value() + self.working.values().copied().map(|s| s.amount).sum()
    }

    /// Consumes the pot and returns the total payout.
    ///
    /// The argument is a vec of the player's hand rankings relative to each other.
    /// The first item in the vec is another vec, and this inner vec is the players that have
    /// equally good and the best hands. The second item is a vec for runner up players, etc.
    ///
    /// For example, passing [[1], [2], [3]] means player 1 had the best hand, followed by 2, and 3
    /// had the worst.
    ///
    /// Passing [[1, 2], [3]] means player 1 and 2 had the best hands and they are equal. 3 had the worst.
    ///
    /// **Only provide players that are still eligible to win (part of) the pot**. Do not include
    /// players that have folded. The reason for including more than just the best player(s) is to
    /// be able to handle side pots. This is also why this function returns a HashMap of players
    /// and their respective winnings.
    ///
    /// # Returns
    ///
    /// HashMap of players and the amount they should be awared from the pot(s).
    pub(crate) fn payout(self, ranked_players: &[Vec<PlayerId>]) -> HashMap<PlayerId, Currency> {
        let (hm, _) = self.payout_with_log(ranked_players);
        hm
    }

    /// Like payout function, but also provides the log of actions we saw and took.
    pub(crate) fn payout_with_log(
        mut self,
        ranked_players: &[Vec<PlayerId>],
    ) -> (HashMap<PlayerId, Currency>, Vec<LogItem>) {
        // In case caller didn't call finalize_round() after the last betting round, do it for them.
        if !self.working.is_empty() {
            self.finalize_round();
        }
        assert!(self.working.is_empty());

        let mut hm: HashMap<PlayerId, Currency> = HashMap::new();
        // Ha! Made you look. All the hard work is done in each inner pot, and the results simply
        // merged together here.
        for (pot_n, pot) in self.settled.into_iter().enumerate() {
            let hm_n = pot.payout(ranked_players);
            self.log.push(LogItem::Payouts(Some(pot_n), hm_n.clone()));
            crate::util::merge_hashmap(&mut hm, hm_n);
        }
        self.log.push(LogItem::Payouts(None, hm.clone()));
        (hm, self.log)
    }

    /// Record that a player has made a bet. The player's **total** bet is to be provided. I.e. if
    /// in a single betting round a player Bet(10) and then Call(30) (due to another player
    /// raising), give this function Call(30), not Call(20).
    pub(crate) fn bet(&mut self, player: PlayerId, action: BetAction) {
        self.log.push(LogItem::Bet(player, action));
        let stake: Stake = match action {
            BetAction::Check | BetAction::Fold => {
                return;
            }
            BetAction::Call(v) | BetAction::Bet(v) | BetAction::Raise(v) => (false, v),
            BetAction::AllIn(v) => (true, v),
        }
        .into();
        self.working.insert(player, stake);
    }
}

impl Default for Pot {
    fn default() -> Self {
        Pot {
            // With no all ins and going all the way to showdown, 3 is the expected number.
            // It's not a big deal if this isn't perfectly accurate. It's just a guess to usually
            // avoid reallocation. It can/will be more if people go all in.
            settled: Vec::with_capacity(3),
            working: HashMap::default(),
            log: Vec::new(),
        }
    }
}

impl Default for InnerPot {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            max_in: None,
        }
    }
}

#[cfg(test)]
mod test_payout {
    use super::*;

    #[test]
    fn simple_single_winner() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Call(5.into()));
        p.bet(3, BetAction::Call(5.into()));
        p.finalize_round();
        let payout = p.payout(&vec![vec![1]]);
        assert_eq!(payout[&1], 15.into());
    }

    #[test]
    fn simple_multi_winner() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Call(5.into()));
        p.bet(3, BetAction::Call(5.into()));
        p.finalize_round();
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8.into());
        assert_eq!(payout[&2], 7.into());

        // it is not possible for the 3rd person to be in for more than the others like this, but
        // the pot does its best to function anyway. Garbage in => garbage out. It's the caller's
        // fault for not knowing how Texas Holdem works.
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Bet(5.into()));
        p.bet(3, BetAction::Bet(6.into()));
        p.finalize_round();
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8.into());
        assert_eq!(payout[&2], 8.into());
    }

    #[test]
    fn three_way_tie() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5.into()));
        p.bet(2, BetAction::Bet(5.into()));
        p.bet(3, BetAction::Bet(5.into()));
        p.finalize_round();
        let payout = p.payout(&vec![vec![1, 2, 3]]);
        dbg!(&payout);
        assert_eq!(payout[&1], 5.into());
        assert_eq!(payout[&2], 5.into());
        assert_eq!(payout[&3], 5.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {}

    #[test]
    fn all_in_blind() {
        let mut p = Pot::default();
        p.bet(1, BetAction::AllIn(5.into()));
        p.bet(2, BetAction::Bet(10.into()));
        p.bet(3, BetAction::AllIn(8.into()));
        p.finalize_round();
        dbg!(&p);
        let payout = p.payout(&vec![vec![1], vec![2, 3]]);
        dbg!(&payout);
        // 5 from each player, 8 remains (5 from p2's call and 3 from p3's allin)
        assert_eq!(payout[&1], 15.into());
        // a second side pot containing 6 (3 for p3's all in, and 3 from p2's call) exists. p2 and
        // p3 tied, so they split it.
        // p2 has 3 and p3 has 3.
        // The final pot has just p2 and their remaining 2. They get that whole pot.
        // p2 has 3+2 and p3 has 3 still.
        assert_eq!(payout[&2], 5.into());
        assert_eq!(payout[&3], 3.into());
    }

    #[test]
    fn side_pot_payout() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10.into()));
        p.bet(2, BetAction::AllIn(5.into()));
        p.bet(3, BetAction::Bet(10.into()));
        p.finalize_round();
        dbg!(&p);
        let payout = p.payout(&vec![vec![2], vec![1, 3]]);
        assert_eq!(payout[&2], 15.into());
        assert_eq!(payout[&1], 5.into());
        assert_eq!(payout[&3], 5.into());
    }

    #[test]
    fn overflowing_side_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10.into()));
        p.bet(2, BetAction::AllIn(5.into()));
        p.bet(3, BetAction::AllIn(3.into()));
        p.finalize_round();
        dbg!(&p);
        let payout = p.payout(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        assert_eq!(payout[&3], 9.into());
        assert_eq!(payout[&2], 4.into());
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 5.into());
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
        let (payout, log) = p.payout_with_log(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        for log_item in &log {
            println!("{}", log_item);
        }
        assert_eq!(payout[&3], 39.into());
        assert_eq!(payout[&2], 16.into());
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 4.into());
    }

    #[test]
    /// bet, call, and raise are all semantically the same as far as the pot is concerned.
    fn bet_call_raise() {
        fn helper(p: Pot) {
            assert_eq!(p.settled.len(), 1);
            let ip = &p.settled[0];
            assert_eq!(ip.players.len(), 3);
            for v in ip.players.values() {
                assert_eq!(v.amount, 5.into());
            }
            assert_eq!(ip.max_in, None);
            dbg!(&p);
            let payout = p.payout(&vec![vec![1]]);
            assert_eq!(payout[&1], 15.into());
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

        assert_eq!(p.settled.len(), 3);

        let pot = &p.settled[0];
        assert_eq!(pot.players.len(), 3);
        for v in pot.players.values() {
            assert_eq!(v.amount, 15.into());
        }
        assert_eq!(pot.max_in, None);

        let pot = &p.settled[1];
        assert_eq!(pot.players.len(), 3);
        for v in pot.players.values() {
            assert_eq!(v.amount, 50.into());
        }
        assert_eq!(pot.max_in, Some(50.into()));

        let pot = &p.settled[2];
        assert_eq!(pot.players.len(), 1);
        for v in pot.players.values() {
            assert_eq!(v.amount, 450.into());
        }
        assert_eq!(pot.max_in, None);
    }

    #[test]
    fn all_all_in() {
        let mut p = Pot::default();
        p.bet(1, BetAction::AllIn(5.into()));
        p.bet(2, BetAction::AllIn(15.into()));
        p.bet(3, BetAction::AllIn(45.into()));
        p.finalize_round();
        dbg!(&p);
        assert_eq!(p.settled.len(), 3);

        let pot = &p.settled[0];
        assert_eq!(pot.players.len(), 3);
        assert_eq!(pot.max_in, Some(5.into()));
        let pot = &p.settled[1];
        assert_eq!(pot.players.len(), 2);
        assert_eq!(pot.max_in, Some(10.into()));
        let pot = &p.settled[2];
        assert_eq!(pot.players.len(), 1);
        assert_eq!(pot.max_in, Some(30.into()));

        let mut p = Pot::default();
        p.bet(1, BetAction::AllIn(45.into()));
        p.bet(2, BetAction::AllIn(15.into()));
        p.bet(3, BetAction::AllIn(5.into()));
        p.finalize_round();
        dbg!(&p);
        assert_eq!(p.settled.len(), 3);

        let pot = &p.settled[0];
        assert_eq!(pot.players.len(), 3);
        assert_eq!(pot.max_in, Some(5.into()));
        let pot = &p.settled[1];
        assert_eq!(pot.players.len(), 2);
        assert_eq!(pot.max_in, Some(10.into()));
        let pot = &p.settled[2];
        assert_eq!(pot.players.len(), 1);
        assert_eq!(pot.max_in, Some(30.into()));
    }
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
