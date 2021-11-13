table! {
    accounts (account_id) {
        account_id -> Integer,
        account_name -> Text,
        api_key -> Text,
        is_admin -> SmallInt,
    }
}

table! {
    game_tables (table_id) {
        table_id -> Integer,
        table_name -> Text,
    }
}

table! {
    money_log (transaction_id) {
        transaction_id -> Nullable<Integer>,
        account_id -> Integer,
        monies -> Integer,
        execution_time -> Nullable<Timestamp>,
        reason -> Text,
    }
}

table! {
    player_meta (account_id) {
        account_id -> Integer,
        player_name -> Text,
        email -> Nullable<Text>,
    }
}

table! {
    seated (table_id, account_id) {
        table_id -> Integer,
        account_id -> Integer,
    }
}

table! {
    settled_accounts (account_id) {
        account_id -> Integer,
        monies -> Integer,
    }
}

table! {
    table_meta (table_id) {
        table_id -> Integer,
        table_state -> Integer,
        hand_num -> Integer,
        buy_in -> Integer,
        small_blind -> Integer,
    }
}

joinable!(money_log -> accounts (account_id));
joinable!(player_meta -> accounts (account_id));
joinable!(seated -> accounts (account_id));
joinable!(seated -> game_tables (table_id));
joinable!(settled_accounts -> accounts (account_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    game_tables,
    money_log,
    player_meta,
    seated,
    settled_accounts,
    table_meta,
);
