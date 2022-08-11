use actix_web::{get, web, HttpResponse, Responder};
use identicon_rs::Identicon;

enum ImageType {
    Png,
    Jpeg,
}

fn generate_image(image_type: ImageType, path: web::Path<String>) -> impl Responder {
    let identicon_string = path.into_inner();
    let identicon = Identicon::new(&identicon_string);

    match image_type {
        ImageType::Png => {
            let data = identicon.export_png_data().unwrap();
            HttpResponse::Ok().content_type("image/png").body(data)
        }
        ImageType::Jpeg => {
            let data = identicon.export_jpeg_data().unwrap();
            HttpResponse::Ok().content_type("image/jpeg").body(data)
        }
    }
}

#[get("/{name}")]
pub async fn generate_png_raw_path(path: web::Path<String>) -> impl Responder {
    generate_image(ImageType::Png, path)
}

#[get("/{name}.png")]
pub async fn generate_png_path(path: web::Path<String>) -> impl Responder {
    generate_image(ImageType::Png, path)
}

#[get("/{name}.jpg")]
pub async fn generate_jpg_path(path: web::Path<String>) -> impl Responder {
    generate_image(ImageType::Jpeg, path)
}

#[get("/{name}.jpeg")]
pub async fn generate_jpeg_path(path: web::Path<String>) -> impl Responder {
    generate_image(ImageType::Jpeg, path)
}
