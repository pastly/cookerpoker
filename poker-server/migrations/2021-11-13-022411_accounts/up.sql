-- Your SQL goes here
CREATE TABLE accounts (
    account_id INTEGER NOT NULL PRIMARY KEY,
    account_name TEXT NOT NULL,
    api_key TEXT,
    is_admin SMALLINT NOT NULL DEFAULT FALSE
);

CREATE TABLE settled_accounts (
    account_id INTEGER NOT NULL PRIMARY KEY,
    monies INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES accounts (account_id)
);

CREATE TABLE player_meta (
    account_id INTEGER NOT NULL PRIMARY KEY,
    player_name TEXT NOT NULL DEFAULT "anonymouse",
    email TEXT,
    FOREIGN KEY (account_id)
        REFERENCES accounts (account_id)
);

CREATE TABLE game_tables (
    table_id INTEGER NOT NULL PRIMARY KEY,
    table_name TEXT NOT NULL
);

CREATE TABLE table_meta (
    table_id INTEGER NOT NULL PRIMARY KEY,
    table_state INTEGER NOT NULL DEFAULT 0,
    hand_num INTEGER NOT NULL DEFAULT 0,
    buy_in INTEGER NOT NULL,
    small_blind INTEGER NOT NULL
);

CREATE TABLE seated (
    table_id INTERGE NOT NULL,
    account_id INTEGER NOT NULL,
    PRIMARY KEY(table_id, account_id)
    FOREIGN KEY (account_id)
        REFERENCES accounts (account_id),
        FOREIGN KEY (table_id)
        REFERENCES game_tables (table_id)
);