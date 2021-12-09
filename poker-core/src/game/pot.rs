use super::players::PlayerId;
use super::BetAction;
use std::collections::HashMap;

/// Handles all pot related operations.
/// Only tracks monies committed to the pot.
/// As such, does no error handling and cannot fail.
/// Parent must validate player has enough monies, and track the state of the betting round.
#[derive(Debug)]
pub struct Pot {
    players_in: HashMap<PlayerId, i32>,
    max_in: i32,
    side_pot: Option<Box<Pot>>,
    is_settled: bool,
}

impl Pot {
    /// Returns the total value in this pot
    /// Not particularily useful due to each betting round spinning off a side pot
    pub fn value(&self) -> i32 {
        self.players_in.values().sum()
    }

    pub fn total_value(&self) -> i32 {
        let mut v = self.players_in.values().sum();
        if let Some(x) = self.side_pot.as_ref() {
            v += x.total_value();
        }
        v
    }

    fn overflowing_add(&mut self, player: PlayerId, amount: i32) {
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

    fn update_max(&mut self, new_max: i32) {
        use std::cmp::Ordering;
        if self.is_settled {
            self.side_pot().update_max(new_max);
        } else {
            if new_max == i32::MAX || new_max < 1 {
                return;
            }
            match new_max.cmp(&self.max_in) {
                Ordering::Greater => self.side_pot().update_max(new_max),
                Ordering::Less => {
                    let ov = self.max_in;
                    self.max_in = new_max;
                    if ov != i32::MAX {
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
    pub fn payout(self, ranked_hands: &[Vec<PlayerId>]) -> HashMap<PlayerId, i32> {
        let mut hm: HashMap<PlayerId, i32> = HashMap::new();
        let value = self.value();
        let mut paid_out = false;
        for best_hand in ranked_hands {
            let hands_in = self.num_players_in(best_hand);
            // Prevents divide by zero below
            if hands_in == 0 {
                continue;
            }
            let payout = value / self.num_players_in(best_hand) as i32;
            for player in best_hand.iter() {
                if self.players_in.contains_key(player) {
                    hm.insert(*player, payout);
                    paid_out = true;
                    if best_hand.len() > 1 && value % 2 == 1 {
                        // TODO Randomize
                        hm.insert(best_hand[0], payout + 1);
                    }
                }
            }
            if paid_out {
                break;
            }
        }
        assert_eq!(hm.values().sum::<i32>(), self.value());
        if let Some(x) = self.side_pot {
            crate::util::merge_hashmap(&mut hm, x.payout(ranked_hands));
        }
        hm
    }

    /// Takes the players TOTAL bet. I.e. Bet(10), Call(20) = bet of 20.
    /// As such, parent must track the current betting round.
    pub fn bet(&mut self, player: PlayerId, action: BetAction) -> i32 {
        use std::cmp::Ordering;
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
                BetAction::Bet(v) | BetAction::Call(v) => v,
                // Folds and calls have no effect on the pot.
                _ => return 0,
            };
            self.overflowing_add(player, value - ov);
            0
        }
    }
}

impl Default for Pot {
    fn default() -> Self {
        Pot {
            players_in: HashMap::new(),
            max_in: i32::MAX,
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
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Call(5));
        p.bet(3, BetAction::Call(5));
        let payout = p.payout(&vec![vec![1]]);
        assert_eq!(payout[&1], 15);
    }

    #[test]
    fn multi_winners() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(5));
        p.bet(3, BetAction::Bet(5));
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 7);

        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(5));
        p.bet(3, BetAction::Bet(6));
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 8);
    }

    #[test]
    fn over_bet() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(5));
        p.bet(3, BetAction::Bet(6));
        let payout = p.payout(&vec![vec![1, 2], vec![3]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 7);
        assert_eq!(payout[&3], 1);
    }

    #[test]
    fn all_in_blind() {
        let mut p = Pot::default();
        p.bet(1, BetAction::AllIn(5));
        p.bet(2, BetAction::Bet(10));
        p.bet(3, BetAction::AllIn(8));
        dbg!(&p);
        let payout = p.payout(&vec![vec![1], vec![2, 3]]);
        dbg!(&payout);
        assert_eq!(payout[&1], 15);
        assert_eq!(payout[&2], 5);
        assert_eq!(payout[&3], 3);
    }

    #[test]
    fn side_pot_payout() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10));
        p.bet(2, BetAction::AllIn(5));
        p.bet(3, BetAction::Bet(10));
        let payout = p.payout(&vec![vec![2], vec![1, 3]]);
        assert_eq!(payout[&2], 15);
        assert_eq!(payout[&1], 5);
        assert_eq!(payout[&3], 5);
    }

    #[test]
    fn overflowing_side_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10));
        p.bet(2, BetAction::AllIn(5));
        p.bet(3, BetAction::AllIn(3));
        dbg!(&p);
        let payout = p.payout(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        assert_eq!(payout[&3], 9);
        assert_eq!(payout[&2], 4);
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 5);
    }

    #[test]
    fn multi_round_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Call(5));
        p.bet(3, BetAction::Call(5));
        p.finalize_round();
        // 5,5,5 = 15 in pot
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(10));
        p.bet(3, BetAction::AllIn(8));
        p.bet(1, BetAction::Call(10));
        p.finalize_round();
        // 15 + 8,8,8 + 2,2 = 43 in pot
        p.bet(1, BetAction::Bet(10));
        p.bet(2, BetAction::AllIn(6));
        // 43 + 6,6 + 4 = 59 in pot
        dbg!(&p);
        let payout = p.payout(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        assert_eq!(payout[&3], 39);
        assert_eq!(payout[&2], 16);
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 4);
    }
}
