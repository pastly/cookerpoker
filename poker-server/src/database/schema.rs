table! {
    accounts (id) {
        id -> Integer,
        account_name -> Text,
        api_key -> Text,
        is_admin -> SmallInt,
        monies -> Integer,
    }
}

table! {
    game_tables (id) {
        id -> Integer,
        table_owner -> Integer,
        table_type -> SmallInt,
        table_name -> Text,
        table_state -> SmallInt,
        hand_num -> Integer,
        buy_in -> Integer,
        small_blind -> Integer,
    }
}

table! {
    money_log (id) {
        id -> Nullable<Integer>,
        account_id -> Integer,
        made_by -> Integer,
        monies -> Integer,
        execution_time -> Nullable<Timestamp>,
        reason -> Text,
    }
}

table! {
    player_meta (account_id) {
        account_id -> Integer,
        player_name -> Nullable<Text>,
        email -> Nullable<Text>,
    }
}

table! {
    seated (table_id, account_id) {
        table_id -> Integer,
        account_id -> Integer,
    }
}

joinable!(game_tables -> accounts (table_owner));
joinable!(player_meta -> accounts (account_id));
joinable!(seated -> accounts (account_id));
joinable!(seated -> game_tables (table_id));

allow_tables_to_appear_in_same_query!(accounts, game_tables, money_log, player_meta, seated,);
