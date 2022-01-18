[issue 41]: https://github.com/pastly/cookerpoker/issues/41
[issue 42]: https://github.com/pastly/cookerpoker/issues/42

# Game log plans

The general existing idea and plan forward is 

- the GameInProgress in poker-core learns about player actions via its `pub fn`
  interface, and
- it produces a log of events that can be used to replay a game and are sent to
  players to update them on game state.

The details on how/where the logs are stored is currently being worked out.

This document explains the current state of things, explains why the current
state doesn't work, and explores alternative ways forward.

Aside: "logs", "log items", "events", etc. are used interchangeable for no good
reason.

## Current state

Calls to GameInProgress's `pub fn`s, for example `bet()`, return a list of log
item that explain how the game progressed in response to the function call. For
example, a call to `bet()` may just return a log item stating that the bet was
made, but if this bet was the end of the pre-flop betting round, it would
return that log item *and* a log item for the flop's cards.

## Problems with current state

### `PocketsDealt`

`PocketsDealt(HashMap<PlayerId, [Card; 2]>)` is one of those log items.
However, as [issue 41][] points out, there's no time we want all pockets known
in a log item:

- Not in a log on disk/in the db: an admin could easily access the disk/db to
  cheat.
- Not sent to client software: it shouldn't be trusted to hide other players'
  pockets.

The map of players to their pockets should be kept only in memory as this
sufficiently raises the bar to prevent malicious admins from cheating.
But what code is responsible for storing this is a question.

### GameInProgress not recording logs

GameInProgress currently immediately returns logs as they are generated. This
may be okay: a reasonable implementation can expect the user of GameInProgress
to maintain logs and disperse them to clients as necessary.

The thing that's considered an issue here is: addressing the `PocketsDealt`
issue likely means GameInProgress holds the player => pocket map for the
duration of the hand. If it's giving its caller log items as soon as they are
generated but withholds the pocket map, the caller has no way of informing
players of their pockets. We further don't consider providing the caller with
the pocket map directly as tenable: pockets are too sensitive of information to
be leaked outside of poker-core willy-nilly.

## Way forward

GameInProgress shall be changed (back) to storing logs as they are generated.
For simplicity's sake, its `pub fn`s such as `bet()` will *not* return logs
that are generated as a result of the call; instead it is the caller's
responsibility to call the `logs_since()` function (discussed momentarily)
immediately after the call if they want any fresh logs.

In support of [issue 42][], GameInProgress will store a small number of full
hands' logs (considered "live"), not just the current hand. "Live" logs will
migrate to "archive" logs also maintained by GameInProgress. The number of
hands' logs to keep as live/archive may not be configurable as part of
implementing this document.

As part of implementing the changes in this document, no logs will be written
to the database. It is understood, however, that once we get to the point of
writing game logs to the database, that players' pockets will not be a part of
those logs.

The `PocketsDealt(_)` log item with the pocket map will be replaced with a
`PocketDealt(PlayerId, [Card; 2])` log item (singular). When pockets are dealt,
one of these will be logged for each player that receives cards. These are the
log items that never touch the database.

GameInProgress will keep the pocket map in memory for the duration of the
current hand.

GameInProgress will gain a `pub fn logs_since(usize, PlayerId) ->
(Vec<LogItem>, usize)`. The usize argument is the index of the most recent log
the caller knows about. The PlayerId argument is the player that the logs
should be tailored for, i.e. all `PocketsDealt(_)` for other players will not
be returned.  The vec return value is the logs, and the usize return value is
the index of the last live log item (last live log item known about, whether or
not it is part of the returned vec. AKA it's `live_logs.len()-1`).

An optional feature as part of implementing this document: GameInProgress will
gain a `pub fn reveal(PlayerId)` to be called when a player voluntarily reveals
their hand (and called automatically at showdown for the player that's required
to reveal their hand, though this may not be implemented at first). In response
to this function call GameInProgress will append a new type of log item to its
log `Reveal(PlayerId, [Card; 2])`.  The cards will come from GameInProgress's
pocket map in memory in order to not have to trust the caller.

### How `PocketsDealt` is addressed

It no longer exists.  `PocketDealt(_)` replaces it. The database or textual log
files never have this event written to them. Players only receive their own
PocketDealt log items.

### How not recording logs is addressed

GameInProgress is modified to record logs instead of returning the latest log
items.
