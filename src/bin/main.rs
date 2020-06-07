use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use jobq::server::Server;
use jobq::settings::Settings;

#[tokio::main]
async fn main() {
    let settings = Settings::get();

    // Set up the r2d2 connection pool
    let manager = ConnectionManager::<PgConnection>::new(&settings.get_database_url());
    let pool = Pool::builder()
        .max_size(settings.database.pool_size)
        .build(manager)
        .unwrap_or_else(|_| panic!("Error connecting to {}", settings.get_database_url()));

    // Get the connection
    let conn = pool.get().unwrap();

    println!("Starting server at {:?}", settings.server.bind);
    let server = Server::new(settings.server.bind);

    server.run().await;
}
