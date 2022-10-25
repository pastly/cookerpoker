"use strict";
const DEAL_CARD_ANIMATE_DUR = 150;
const DEAL_CARD_WAIT_DUR = 100;

class Ani {
    constructor(animate_dur, wait_dur) {
        this.animate_dur = animate_dur;
        this.wait_dur = wait_dur;
    }
}

class DealCardAni extends Ani {
    constructor(pocket_n, card_n, char) {
        super(DEAL_CARD_ANIMATE_DUR, DEAL_CARD_WAIT_DUR);
        this.pocket_n = pocket_n;
        this.card_n = card_n;
        this.char = char;
    }

    animate() {
        let container = document.getElementById(ID_TABLE_CONTAINER);
        let cont_w = container.clientWidth;
        let cont_h = container.clientHeight;
        let pocket = document.getElementById(`${IDPREFIX_POCKET}${this.pocket_n}`);
        let pocket_pos_x = pocket.style.left.substring(0, pocket.style.left.length-2);
        let pocket_pos_y = pocket.style.top.substring(0, pocket.style.top.length-2);
        let item = pocket.getElementsByClassName(`${IDPREFIX_CARD}${this.card_n}`)[0];
        let card_w = item.clientWidth;
        let card_h = item.clientHeight;
        item.innerText = this.char;
        item.animate([
          { transform: `translate(${cont_w/2-pocket_pos_x-card_w/2}px, ${cont_h/2-pocket_pos_y-card_h/2}px)` },
          { transform: `translate(0px, 0px)` }
        ], {
          duration: this.animate_dur,
          iterations: 1,
        });
        return this.wait_dur;
    }
}
