export {
  initialize_table,
  pocket_coords,
};

import * as vars from "./table_vars.js";
//import * as animate from "./animate.js";

export let TABLE_SIZE = null;
export let POCKET_SIZE = null;
export let COMMUNITY_SIZE = null;
export let CARD_SIZE = null;
export let POT_SIZE = null;
export let WAGER_SIZE = null;

function initialize_table() {
  calculate_sizes();
  create_elements();
}

function calculate_sizes() {
  let table = document.getElementById(vars.ID_TABLE_CONTAINER);
  let child = null;
  while ((child = table.lastChild) != null) {
    table.removeChild(child);
  }
  create_pockets(1, false);
  create_wagers(1, false);
  create_community(true);
  create_pot(true);
  let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}0`);
  let wager = document.getElementById(`${vars.IDPREFIX_POCKET}0-wager`);
  let community = document.getElementById(vars.ID_COMMUNITY);
  let pot = document.getElementById(vars.ID_POT);
  let card = document.createElement("p");
  card.classList.add("card");
  card.classList.add(vars.IDPREFIX_CARD + "0");
  card.innerText = "🃅";
  pocket.appendChild(card);
  POCKET_SIZE = {w: pocket.clientWidth, h: pocket.clientHeight};
  WAGER_SIZE = {w: wager.clientWidth, h: wager.clientHeight};
  TABLE_SIZE = {w: table.clientWidth, h: table.clientHeight};
  COMMUNITY_SIZE = {w: community.clientWidth, h: community.clientHeight};
  CARD_SIZE = {w: card.clientWidth, h: card.clientHeight};
  POT_SIZE = {w: pot.clientWidth, h: pot.clientHeight};
  child = null;
  while ((child = community.lastChild) != null) {
    community.removeChild(child);
  }
  while ((child = pocket.lastChild) != null) {
    pocket.removeChild(child);
  }
  while ((child = table.lastChild) != null) {
    table.removeChild(child);
  }
}

function create_elements() {
  create_pockets(vars.N_POCKETS, true);
  place_pockets(vars.N_POCKETS);
  create_wagers(vars.N_POCKETS, true);
  place_wagers(vars.N_POCKETS);
  create_community(false);
  place_community();
  create_pot(false);
  place_pot();
}

function pocket_coords(n) {
  let container = document.getElementById(vars.ID_TABLE_CONTAINER);
  if (n < 0 || n > 5) {
    return [0, 0];
  }
  let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${n}`);
  let x = 0;
  let y = 0;
  if (n == 0 || n == 4) {
    x = TABLE_SIZE.w / 3 - POCKET_SIZE.w / 1.5;
    if (n == 0) {
      y = 0;
    } else {
      y = TABLE_SIZE.h - POCKET_SIZE.h;
    }
  } else if (n == 1 || n == 3) {
    x = 2 * TABLE_SIZE.w / 3 - POCKET_SIZE.w / 1.5;
    if (n == 1) {
      y = 0;
    } else {
      y = TABLE_SIZE.h - POCKET_SIZE.h;
    }
  } else if (n == 2) {
    x = TABLE_SIZE.w - POCKET_SIZE.w;
    y = TABLE_SIZE.h / 2 - POCKET_SIZE.h / 2;
  } else {
    x = 0;
    y = TABLE_SIZE.h / 2 - POCKET_SIZE.h / 2;
  }
  return [x, y];
}
function create_pocket(i, hidden) {
  let table = document.getElementById(vars.ID_TABLE_CONTAINER);
  let pocket = document.createElement("div");
  pocket.id = `${vars.IDPREFIX_POCKET}${i}`;
  pocket.classList.add("pocket");
  if (hidden) {
    pocket.classList.add("hide");
  } else {
    pocket.classList.remove("hide");
  }
  table.appendChild(pocket);

  let name = document.createElement("span");
  name.classList.add("name");
  name.innerText = `Player ${i}`;
  let stack = document.createElement("span");
  stack.classList.add("stack");
  stack.innerText = "1000";
  pocket.appendChild(name);
  pocket.appendChild(stack);
  let c1 = document.createElement("p");
  c1.classList.add("card");
  c1.classList.add(vars.IDPREFIX_CARD + "0");
  //c1.innerText = "🃅";
  let c2 = document.createElement("p");
  c2.classList.add("card");
  c2.classList.add(vars.IDPREFIX_CARD + "1");
  //c2.innerText = "🂾";
  pocket.appendChild(c1);
  pocket.appendChild(c2);
}
function create_pockets(n, hidden) {
  for (let i = 0; i < n; i++) {
    create_pocket(i, hidden);
  }
}
function place_pocket(i) {
    let pocket = document.getElementById(`${vars.IDPREFIX_POCKET}${i}`);
    const [x, y] = pocket_coords(i);
    pocket.style.left = `${x}px`;
    pocket.style.top = `${y}px`;
}
function place_pockets(n) {
  for (let i = 0; i < n; i++) {
    place_pocket(i);
  }
}
function create_wager(i, hidden) {
  let table = document.getElementById(vars.ID_TABLE_CONTAINER);
  let id = `${vars.IDPREFIX_POCKET}${i}-wager`;
  let wager = document.createElement("div");
  wager.innerText = "dummy text";
  wager.id = id;
  wager.classList.add("wager");
  if (hidden) {
    wager.classList.add("hide");
  } else {
    wager.classList.remove("hide");
  }
  console.log(`create wager ${i}`);
  table.appendChild(wager);
}
function create_wagers(n, hidden) {
  for (let i = 0; i < n; i++) {
    create_wager(i, hidden);
  }
}
function place_wager(i) {
  let wager = document.getElementById(`${vars.IDPREFIX_POCKET}${i}-wager`);
  const [pocket_x, pocket_y] = pocket_coords(i);
  let to_left = pocket_x < TABLE_SIZE.w / 3;
  let x = pocket_x;
  let y = pocket_y;
  if (pocket_x < TABLE_SIZE.w / 6) {
    x = pocket_x + POCKET_SIZE.w;
  } else if (pocket_x < TABLE_SIZE.w / 2) {
    x = pocket_x + POCKET_SIZE.w;
  } else if (pocket_x > 5 * TABLE_SIZE.w / 6) {
    x = pocket_x - WAGER_SIZE.w;
  }
  if (pocket_y < TABLE_SIZE.h / 4) {
    y = POCKET_SIZE.h;
  } else if (pocket_y < TABLE_SIZE.h / 2) {
    y = pocket_y + POCKET_SIZE.h / 2;
  } else {
    y = pocket_y - WAGER_SIZE.h;
  }
  wager.style.left = `${x}px`;
  wager.style.top = `${y}px`;
}
function place_wagers(n) {
  for (let i = 0; i < n; i++) {
    place_wager(i);
  }
}
function create_community(with_fake_cards) {
  let table = document.getElementById(vars.ID_TABLE_CONTAINER);
  let comm = document.createElement("div");
  comm.id = vars.ID_COMMUNITY;
  for (let i = 0; i < 5; i++) {
    let card = document.createElement("p");
    card.classList.add("card");
    if (with_fake_cards) {
      card.innerText = "🃅";
    }
    comm.appendChild(card);
  }
  table.appendChild(comm);
}
function place_community() {
  let comm = document.getElementById(vars.ID_COMMUNITY);
  let x = TABLE_SIZE.w / 2 - COMMUNITY_SIZE.w / 2;
  let y = TABLE_SIZE.h / 2 - COMMUNITY_SIZE.h / 2;
  //let y = POCKET_SIZE.h;
  comm.style.left = `${x}px`;
  comm.style.top = `${y}px`;
}
function create_pot(with_fake_text) {
  let table = document.getElementById(vars.ID_TABLE_CONTAINER);
  let pot = document.createElement("div");
  pot.id = vars.ID_POT;
  if (with_fake_text) {
    pot.innerText = "Pot: 12345 | Side Pot: 12345 | Side Pot: 12345";
  }
  table.appendChild(pot);
}
function place_pot() {
  let comm = document.getElementById(vars.ID_COMMUNITY);
  let pot = document.getElementById(vars.ID_POT);
  let x = TABLE_SIZE.w / 2 - COMMUNITY_SIZE.w / 2;
  let y = TABLE_SIZE.h / 2 + COMMUNITY_SIZE.h / 2;
  pot.style.left = `${x}px`;
  pot.style.top = `${y}px`;
}
