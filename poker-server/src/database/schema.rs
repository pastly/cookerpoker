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

joinable!(player_meta -> accounts (account_id));
joinable!(seated -> accounts (account_id));
joinable!(seated -> game_tables (table_id));
joinable!(settled_accounts -> accounts (account_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    game_tables,
    player_meta,
    seated,
    settled_accounts,
    table_meta,
);
