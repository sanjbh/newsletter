use std::net::TcpListener;

use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: sqlx::PgPool) -> anyhow::Result<Server> {
    // let connection = web::Data::new(&db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(web::Data::new(db_pool.clone()))
    })
    // .bind(&address)?
    .listen(listener)?
    .run();
    Ok(server)
}
