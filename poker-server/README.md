# Server Setup
1. Install Sqlite (Linux/Mac)
2. Install Diesel cli (`cargo install diesel_cli --no-default-features --features sqlite`. Windows use ``--features sqlite-bundled`)
3. If deploying to prod, back up the database.
4. `diesel migration run`. Supply `--database_url=` as appropriate. "test.db" for testing or target the production DB file. 
5. generate sqllite3.lib and set SQLITE3_LIB_DIR  anyway because you need it to compile the server.
6. Make sure sqlite3.dll is in $PATH
7. ???
8. `cargo run`
9. Lose money to Matt

# TODO
1. Define table state magic (game not started, game started but accepting players, game full, game closed)