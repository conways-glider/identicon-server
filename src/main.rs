use actix_web::{App, HttpServer};

mod image;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(image::generate_jpeg_path)
            .service(image::generate_jpg_path)
            .service(image::generate_png_path)
            .service(image::generate_png_raw_path)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
