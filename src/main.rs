use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use actix_rt;
use http::header::HeaderValue;  // Import HeaderValue directly

async fn get_health_status(data: web::Data<AppState>) -> HttpResponse {
    let is_database_connected = data.db_conn.is_ok();
    if is_database_connected {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::json!({ "database_connected": is_database_connected }).to_string())
    } else {
        HttpResponse::ServiceUnavailable()
            .content_type("application/json")
            .body(serde_json::json!({ "database_connected": is_database_connected }).to_string())
    }
}

struct AppState {
    db_conn: Result<Pool<Postgres>, sqlx::Error>,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Check if the DATABASE_URL environment variable is set
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("DATABASE_URL environment variable is not set.");
            // Return an HTTP server with a 503 Service Unavailable response
            return HttpServer::new(|| {
                App::new()
                    .service(web::resource("/health")
                        .route(web::get().to(|| HttpResponse::ServiceUnavailable()))
                )
            })
            .bind(("0.0.0.0", 8080))?
            .run()
            .await;
        }
    };

    let db_conn_result = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(2))
        .connect(database_url.as_str())
        .await;

    let app_state = web::Data::new(AppState {
        db_conn: db_conn_result,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::DefaultHeaders::new().header("Cache-Control", "no-cache, no-store, must-revalidate",))
            .service(web::resource("/health")
                .route(web::get().to(get_health_status))
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}




mod tests {
    use super::*;
use actix_web::http::StatusCode;
use actix_rt::System;
use actix_web::test;
    
// #[actix_rt::test]
// async fn test_health_status_ok() {
//     System::new().block_on(async {
//         // Initialize your AppState with a mocked database connection (Result<Pool<Postgres>, sqlx::Error>)
//         let app_state = AppState {
//             db_conn: Ok(Pool::connect("mocked_database_url").await),
//         };

//         // Create and configure your Actix Web app, providing the app_state
//         let mut app = test::init_service(
//             App::new().app_data(web::Data::new(app_state)).service(
//                 web::resource("/health")
//                     .route(web::get().to(get_health_status)),
//             ),
//         )
//         .await;

//         // Create a test request
//         let req = test::TestRequest::get().uri("/health").to_request();

//         // Send the request to the app
//         let resp = test::call_service(&mut app, req).await;

//         // Check the response status code
//         assert_eq!(resp.status(), StatusCode::OK);
//     });
// }



#[actix_rt::test]
async fn test_health_status_unavailable() {
   
    let app_state = AppState {
        db_conn: Err(sqlx::Error::PoolTimedOut),
    };

   
    let mut app = test::init_service(
        App::new().app_data(web::Data::new(app_state)).service(
            web::resource("/health")
                .route(web::get().to(get_health_status)),
        ),
    )
    .await;

    
    let req = test::TestRequest::get().uri("/health").to_request();

    
    let resp = test::call_service(&mut app, req).await;

   
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}
#[actix_rt::test]
async fn test_health_method_unavailable() {
    // Initialize your AppState with a mocked database connection (Result<Pool<Postgres>, sqlx::Error>)
    let app_state = AppState {
        db_conn: Err(sqlx::Error::PoolTimedOut), // Simulate a database connection error
    };

    // Create and configure your Actix Web app, providing the app_state
    let mut app = test::init_service(
        App::new().app_data(web::Data::new(app_state)).service(
            web::resource("/health")
                .route(web::get().to(get_health_status)),
        ),
    )
    .await;

    // Create a test request
    let req = test::TestRequest::get().uri("/healthabc").to_request();

    // Send the request to the app
    let resp = test::call_service(&mut app, req).await;

    // Check the response status code
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

}
