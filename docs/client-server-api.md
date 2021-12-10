# Client/Server API

The WASM poker-client communicates over HTTP with the poker-server. The API is
JSON-based (probably\*). Over Tor, HTTPS will not necessarily be used, but not
over Tor, HTTPS will definitely be used.

Once in a game, the client periodically polls the server for any game updates.
This polling doubles as an implicit ping/keep-alive/I'm-still-here message. If
no new actions have been taken, the server sends back an empty list. If there
are new actions that the server hasn't sent to this client, the server sends
them.

\* Something else may be significantly more compact, and I do not think we are
planning on supporting other clients, so maybe something other than JSON will
be used.

All actions for a game are indexed with a sequence number starting at one
(TODO: start at a random number? Worth the complexity? Probably not.). The
client records the sequence number of the latest (largest) action it last
received. Sequence numbers monotonically increase; no sequence numbers are
skipped. As part of the client's request for any new actions, it sends the
latest sequence number it knows about. The server is recording the game in its
entirety and will send all actions the client has not yet seen.

The client knows when it is time for its user to act; there is no special
message from the server to indicate action is on such-and-such user. Where the
action is is implicit and unambiguous based on the recorded actions. If action
is on a client that is non-responsive, after the configured delay the server
will record and announce the appropriate action for the client (i.e. check if
possible, else fold). If the server must take an action for the client, the
client is further considered to be sitting out for the next hand unless they
reconnect before the next hand is dealt.

## Message: Client->Server poll for new actions

Contents:

- *seq_num* (**required**): The sequence number of the last action the client
  knows about. If the game has just started, this is 0 (the first actual
action's sequence number will be 1).

- *action* (**optional**): The action the client wishes to take. This is either
  null or not specified if the client has not yet decided on an action.

## Message: Server->Client list of new actions

Contents:

- *actions* (**required**): List of actions, in order, that are newer than the
  client's specified sequence number. If there are no new actions, this is
still specified as an empty list.

## Action: epoch

TODO

Indicates a new "page" in game history. A client that joins late expects to be
caught with actions taken since (and including) the most recent epoch. An epoch
contains everything a client needs to know about basic/background game
information.

Contents:

- Each player, their seat index, their name, their ID, and their moneies.
- amount of time client has to make a decision.
- blinds.
- seat index of the dealer, small, and big blind buttons.

## Action: player sit down

TODO

A player sat down. Includes their display name, seat position, ID, and
monies.

## Action: player stand up

TODO

A player stands up. Includes either their seat index or their ID (pick one).

## Action: cards dealt

TODO

A new hand has been started. Includes a list seats that are dealt a pocket, and
the 2 cards this user was dealt.

## Action: bet action

TODO

The user has decided to bet/raise/check/fold, and if applicable, the amount of
monies.

The monies is the total amount. For example, if player A bets 2, B raises to 5,
and A wants to call, A would Call(5), not Call(3).

## Action: community cards

TODO

Three (the flop) or one (turn or river) cards have been added to the community
cards. Includes a list of the cards; it is always a list, even if just one
card.

## Action: reveal

TODO is this needed? Probably

The player reveals their pocket as part of showdown. This is underspecified on
purpose because it is unclear what exactly will be needed at showdown.
