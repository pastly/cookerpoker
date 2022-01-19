-- Your SQL goes here
CREATE TABLE accounts (
    id INTEGER NOT NULL PRIMARY KEY,
    account_name TEXT NOT NULL,
    api_key TEXT NOT NULL UNIQUE,
    is_admin SMALLINT NOT NULL DEFAULT FALSE,
    monies INTEGER NOT NULL DEFAULT 0
);

INSERT INTO accounts (account_name, api_key, is_admin)
VALUES ("test_account", "not_a_real_api_key______________________42", 1);

CREATE TABLE money_log (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    made_by INTEGER NOT NULL,
    monies INTEGER NOT NULL,
    execution_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    reason TEXT NOT NULL,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id),
    FOREIGN KEY(account_id)
        REFERENCES accounts(id)
);

CREATE TABLE player_meta (
    account_id INTEGER NOT NULL PRIMARY KEY,
    player_name TEXT,
    email TEXT,
    FOREIGN KEY (account_id)
        REFERENCES accounts (id)
);

CREATE TABLE game_tables (
    id INTEGER NOT NULL PRIMARY KEY,
    table_owner INTEGER NOT NULL,
    table_type SMALLINT NOT NULL DEFAULT 1,
    table_name TEXT NOT NULL UNIQUE,
    table_state SMALLINT NOT NULL DEFAULT 0,
    hand_num INTEGER NOT NULL DEFAULT 0,
    buy_in INTEGER NOT NULL DEFAULT 500,
    small_blind INTEGER NOT NULL DEFAULT 5,
    FOREIGN KEY (table_owner)
        REFERENCES accounts (id)
);

CREATE TABLE seated (
    table_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    PRIMARY KEY(table_id, account_id)
    FOREIGN KEY (account_id)
        REFERENCES accounts (id),
        FOREIGN KEY (table_id)
        REFERENCES game_tables (id)
);
