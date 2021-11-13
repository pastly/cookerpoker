# Server Setup
1. Install Sqlite (Linux/Mac)
2. Install Diesel cli (`cargo install diesel_cli --no-default-features --features sqlite`. Windows use ``--features sqlite-bundled`)
3. generate sqllite3.lib and set SQLITE3_LIB_DIR  anyway because you need it to compile the server.
4. ???
5. `cargo run`
6. Lose money to Matt

# TODO
1. Define table state magic (game not started, game started but accepting players, game full, game closed)