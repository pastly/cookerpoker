export { animate_next };
export {
    DealCardPocketAni,
    DealCardCommunityAni,
    RevealCardsAni,
    RedrawPocketAni,
    ClearCommunityAni,
    ClearBetsAni,
    ClearPotAni,
    MakeBetAni,
    CollectPotAni,
    PushWinningsAni,
    NextToActAni,
};
export let ANIMATION_QUEUE = new Array();
let ANIMATION_TIMEOUT = 0;

import * as vars from "./table_vars.js";
import * as table from "./table.js";

const INSTANT_ANIMATE_DUR = 0;
const INSTANT_WAIT_DUR = 0;
const DEAL_CARD_POCKET_ANIMATE_DUR = 150;
const DEAL_CARD_POCKET_WAIT_DUR = 100;
const DEAL_CARD_COMMUNITY_ANIMATE_DUR = 150;
const DEAL_CARD_COMMUNITY_WAIT_DUR = 250;
const MAKE_BET_ANIMATE_DUR = 200;
const MAKE_BET_WAIT_DUR = 300;
const COLLECT_POT_ANIMATE_DUR = 200;
const COLLECT_POT_WAIT_DUR = 300;
const PUSH_WINNINGS_ANIMATE_DUR = 250;
const PUSH_WINNINGS_WAIT_DUR = 1250;
const REVEAL_CARDS_ANIMATE_DUR = 250;
const REVEAL_CARDS_WAIT_DUR = 1250;


function animate_next() {
  if (ANIMATION_QUEUE.length < 1) {
    return;
  } else if (ANIMATION_TIMEOUT > 0) {
    return;
  }
  let ani = ANIMATION_QUEUE.shift();
  let wait = ani.animate();
  ANIMATION_TIMEOUT = 0;
  if (ANIMATION_QUEUE.length > 0) {
    ANIMATION_TIMEOUT = setTimeout(() => {
        ANIMATION_TIMEOUT = 0;
        animate_next();
    }, wait);
  }
}

class Ani {
    constructor(animate_dur, wait_dur) {
        this.animate_dur = animate_dur;
        this.wait_dur = wait_dur;
    }
}

class DealCardPocketAni extends Ani {
    constructor(pocket_n, card_n, card) {
        super(DEAL_CARD_POCKET_ANIMATE_DUR, DEAL_CARD_POCKET_WAIT_DUR);
        this.pocket_n = pocket_n;
        this.card_n = card_n;
        this.card = card;
    }

    animate() {
        let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${this.pocket_n}`);
        let pocket_pos_x = pocket.style.left.substring(0, pocket.style.left.length-2);
        let pocket_pos_y = pocket.style.top.substring(0, pocket.style.top.length-2);
        let item = pocket.getElementsByClassName(`${vars.IDPREFIX_CARD}${this.card_n}`)[0];
        let center_x = table.TABLE_SIZE.w / 2 - pocket_pos_x - table.CARD_SIZE.w / 2;
        let center_y = table.TABLE_SIZE.h / 2 - pocket_pos_y - table.CARD_SIZE.h / 2;
        let have_card = this.card != null;
        item.innerText = have_card ? this.card.char() : "🂠";
        item.classList.remove("hide");
        for (let suit of ["club", "diamond", "heart", "spade"]) {
            if (!have_card || suit != this.card.suit()) {
                item.classList.remove(`card-${suit}`);
            } else if (have_card && suit == this.card.suit()) {
                item.classList.add(`card-${suit}`);
            }
        } 
        item.animate([
          { transform: `translate(${center_x}px, ${center_y}px)` },
          { transform: `translate(0px, 0px)` },
        ], {
          duration: this.animate_dur,
          iterations: 1,
        });
        return this.wait_dur;
    }
}

class DealCardCommunityAni extends Ani {
    constructor(card_n, card) {
        super(DEAL_CARD_COMMUNITY_ANIMATE_DUR, DEAL_CARD_COMMUNITY_WAIT_DUR);
        this.card_n = card_n;
        this.card = card;
    }

    animate() {
        let comm = document.getElementById(vars.ID_COMMUNITY);
        let comm_pos_x = comm.style.left.substring(0, comm.style.left.length-2);
        let comm_pos_y = comm.style.top.substring(0, comm.style.top.length-2);
        let item = comm.getElementsByClassName("card")[this.card_n];
        let center_x = table.TABLE_SIZE.w / 2 - comm_pos_x - this.card_n * table.CARD_SIZE.w;
        let center_y = table.TABLE_SIZE.h / 2 - comm_pos_y - table.CARD_SIZE.h / 2;
        item.innerText = this.card.char();
        item.classList.remove("hide");
        for (let suit of ["club", "diamond", "heart", "spade"]) {
            if (suit != this.card.suit()) {
                item.classList.remove(`card-${suit}`);
            } else if (suit == this.card.suit()) {
                item.classList.add(`card-${suit}`);
            }
        }
        item.animate([
          { transform: `translate(${center_x}px, ${center_y}px)` },
          { transform: `translate(0px, 0px)` },
        ], {
            duration: this.animate_dur,
            iterations: 1,
        });
        return this.wait_dur;
    }
}

class RevealCardsAni extends Ani {
    constructor(pocket_n, card0, card1) {
        super(REVEAL_CARDS_ANIMATE_DUR, REVEAL_CARDS_WAIT_DUR);
        this.pocket_n = pocket_n;
        this.cards = [card0, card1];
    }
    animate() {
        let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${this.pocket_n}`);
        for (let i = 0; i < this.cards.length; i++) {
            let card = this.cards[i];
            if (!card) {
                continue;
            }
            let item = pocket.getElementsByClassName(`${vars.IDPREFIX_CARD}${i}`)[0];
            item.innerText = card.char();
            for (let suit of ["club", "diamond", "heart", "spade"]) {
                if (suit != card.suit()) {
                    item.classList.remove(`card-${suit}`);
                } else if (suit == card.suit()) {
                    item.classList.add(`card-${suit}`);
                }
            }
            item.animate([
                { transform: `translate(0px, -10px)` },
                { transform: `translate(0px, 0px)` },
            ], {
                duration: this.animate_dur,
                iterations: 1,
            });
        }
        return this.wait_dur;

    }
}

class RedrawPocketAni extends Ani {
    constructor(pocket_n, name, stack) {
        super(INSTANT_ANIMATE_DUR, INSTANT_WAIT_DUR);
        this.pocket_n = pocket_n;
        this.name = name;
        this.stack = stack;
    }
    animate() {
        let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${this.pocket_n}`);
        pocket.classList.remove("hide");
        pocket.getElementsByClassName("name")[0].innerText = this.name;
        pocket.getElementsByClassName("stack")[0].innerText = this.stack;
        for (let elm of pocket.getElementsByClassName("card")) {
            elm.classList.add("hide");
        }
        let wager_id = `${vars.IDPREFIX_POCKET}${this.pocket_n}-wager`;
        let wager = document.getElementById(wager_id);
        wager.classList.add("hide");
        //console.log(this);
        return this.wait_dur;
    }
}

class ClearCommunityAni extends Ani {
    constructor() {
        super(INSTANT_ANIMATE_DUR, INSTANT_WAIT_DUR);
    }
    animate() {
        let comm = document.getElementById(vars.ID_COMMUNITY);
        for (let card of comm.getElementsByClassName("card")) {
            card.classList.add("hide");
        }
        return this.wait_dur;
    }
}

class ClearBetsAni extends Ani {
    constructor() {
        super(INSTANT_ANIMATE_DUR, INSTANT_WAIT_DUR);
    }
    animate() {
        for (let i = 0; i < vars.N_POCKETS; i++) {
            let wager_id = `${vars.IDPREFIX_POCKET}${i}-wager`;
            let wager = document.getElementById(wager_id);
            wager.classList.add("hide");
        }
        return this.wait_dur;
    }
}

class ClearPotAni extends Ani {
    constructor() {
        super(INSTANT_ANIMATE_DUR, INSTANT_WAIT_DUR);
    }
    animate() {
        let pot = document.getElementById(vars.ID_POT);
        pot.innerText = "";
        return this.wait_dur;
    }
}

class MakeBetAni extends Ani {
    constructor(pocket_n, new_stack, total_wager) {
        super(MAKE_BET_ANIMATE_DUR, MAKE_BET_WAIT_DUR);
        this.pocket_n = pocket_n;
        this.new_stack = new_stack;
        this.total_wager = total_wager;
    }
    animate() {
        let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${this.pocket_n}`);
        if (!pocket) {
            console.log(`In animation, pocket idx ${this.pocket_n} made a bet but doesnt exist`)
            return this.wait_dur;
        }
        pocket.getElementsByClassName("stack")[0].innerText = this.new_stack;
        if (this.total_wager > 0) {
            let wager_id = `${vars.IDPREFIX_POCKET}${this.pocket_n}-wager`;
            let wager = document.getElementById(wager_id);
            wager.innerText = this.total_wager;
            let [pocket_x, pocket_y] = table.pocket_coords(this.pocket_n);
            let wager_x = wager.style.left.substring(0, wager.style.left.length-2);
            let wager_y = wager.style.top.substring(0, wager.style.top.length-2);
            let move_x = wager_x - pocket_x;
            if (wager_x > pocket_x) {
                move_x *= -1;
            }
            let move_y = wager_y - pocket_y;
            if (wager_y > pocket_y) {
                move_y *= -1;
            }
            wager.classList.remove("hide");
            wager.animate([
            { transform: `translate(${move_x}px, ${move_y}px)` },
            { transform: `translate(0px, 0px)` },
            ], {
                duration: this.animate_dur,
                iterations: 1,
            });
        }
        return this.wait_dur;
    }
}

class CollectPotAni extends Ani {
    constructor(pots) {
        super(COLLECT_POT_ANIMATE_DUR, COLLECT_POT_WAIT_DUR);
        this.pots = pots;
    }
    animate() {
        let pot = document.getElementById(vars.ID_POT);
        let pot_x = pot.style.left.substring(0, pot.style.left.length-2);
        let pot_y = pot.style.top.substring(0, pot.style.top.length-2);
        pot.classList.remove("hide");
        for (let i = 0; i < vars.N_POCKETS; i++) {
            let wager_id = `${vars.IDPREFIX_POCKET}${i}-wager`;
            let wager = document.getElementById(wager_id);
            if (wager.classList.contains("hide")) {
                continue;
            }
            let wager_x = wager.style.left.substring(0, wager.style.left.length-2);
            let wager_y = wager.style.top.substring(0, wager.style.top.length-2);
            let offset_x = pot_x - wager_x;
            let offset_y = pot_y - wager_y;
            wager.animate([
                {transform: `translate(0px, 0px)` },
                {transform: `translate(${offset_x}px, ${offset_y}px)` },
            ], {
                duration: this.animate_dur,
                iterations: 1,
            });
            let s = "Pot: ";
            for (let pot of this.pots) {
                s += `${pot} `;
            }
            setTimeout(() => {
                pot.innerText = s;
                wager.classList.add("hide");
                wager.animate([
                    {transform: `translate(0px, 0px)` },
                ], {
                    duration: 1,
                    iterations: 1,
                });
            }, this.animate_dur-1);

        }
        return this.wait_dur;
    }
}

class PushWinningsAni extends Ani {
    constructor(seats, winnings) {
        super(PUSH_WINNINGS_ANIMATE_DUR, PUSH_WINNINGS_WAIT_DUR);
        this.seats = seats;
        this.winnings = winnings;
    }
    animate() {
        let pot = document.getElementById(vars.ID_POT);
        let pot_x = pot.style.left.substring(0, pot.style.left.length-2);
        let pot_y = pot.style.top.substring(0, pot.style.top.length-2);
        pot.classList.add("hide");
        for (let i = 0; i < this.seats.length; i++) {
            let player_n = this.seats[i];
            let amount = this.winnings[i];
            let wager_id = `${vars.IDPREFIX_POCKET}${player_n}-wager`;
            let wager = document.getElementById(wager_id);
            wager.classList.remove("hide");
            wager.innerText = amount;
            let wager_x = wager.style.left.substring(0, wager.style.left.length-2);
            let wager_y = wager.style.top.substring(0, wager.style.top.length-2);
            let offset_x = pot_x - wager_x;
            let offset_y = pot_y - wager_y;
            wager.animate([
                {transform: `translate(${offset_x}px, ${offset_y}px)` },
                {transform: `translate(0px, 0px)` },
            ], {
                duration: this.animate_dur,
                iterations: 1,
            });
        }
        return this.wait_dur;
    }
}

class NextToActAni extends Ani {
    constructor(pocket_n) {
        super(INSTANT_ANIMATE_DUR, INSTANT_WAIT_DUR);
        this.pocket_n = pocket_n;
    }
    animate() {
        for (let i = 0; i < vars.N_POCKETS; i++) {
            let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${i}`);
            if (i == this.pocket_n) {
                pocket.classList.add("next-action");
            } else {
                pocket.classList.remove("next-action");
            }
        }
    }
}
