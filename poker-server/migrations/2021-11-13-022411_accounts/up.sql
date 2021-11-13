-- Your SQL goes here
CREATE TABLE accounts (
    account_id INTEGER NOT NULL PRIMARY KEY,
    account_name TEXT NOT NULL,
    api_key TEXT NOT NULL,
    is_admin SMALLINT NOT NULL DEFAULT FALSE
);

INSERT INTO accounts (account_id, account_name, api_key, is_admin)
VALUES (2147483647, "test_account", "not_a_real_api_key", 1);

CREATE TABLE settled_accounts (
    account_id INTEGER NOT NULL PRIMARY KEY,
    monies INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id)
        REFERENCES accounts (account_id)
);

INSERT INTO settled_accounts (account_id)
VALUES(2147483647);

CREATE TABLE money_log (
    transaction_id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    monies INTEGER NOT NULL,
    execution_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    reason TEXT NOT NULL,
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