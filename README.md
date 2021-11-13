# CookerPoker

NL Texas Hold 'em, but with like, slow cookers or smokers or grills or
something.

This repository holds all the crates in the CookerPoker ecosystem.

Crates are of one of three types (whether or not they're denotated as such in
the repo already):

1. Core game logic (what a card is, how betting works, what hands beat other
   hands, etc.)
2. Web server-side/backend (the web server and what it does: user profiles,
   game API, running a game, connections to a DB, etc.)
3. Web client-side/frontend (compiles to WASM, runs in client browsers)
