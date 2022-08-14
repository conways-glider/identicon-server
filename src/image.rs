use std::path::Path as StdPath;

use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::{extract::Path, response::IntoResponse, Extension};
use identicon_rs::Identicon;

use crate::error::{self, AppError};
use crate::Args;
enum ImageType {
    Png,
    Jpeg,
}

fn generate_image_response(
    args: Args,
    image_type: ImageType,
    input: &str,
) -> Result<Response, AppError> {
    let mut identicon = Identicon::new(input);
    identicon.set_border(args.border);
    identicon.set_size(args.size).map_err(AppError::Identicon)?;
    identicon
        .set_scale(args.scale)
        .map_err(AppError::Identicon)?;
    identicon.set_mirrored(args.mirrored);

    let data = match image_type {
        ImageType::Png => identicon.export_png_data().unwrap(),
        ImageType::Jpeg => identicon.export_jpeg_data().unwrap(),
    };
    let headers = match image_type {
        ImageType::Png => [(header::CONTENT_TYPE, "image/png".to_string())],
        ImageType::Jpeg => [(header::CONTENT_TYPE, "image/jpeg".to_string())],
    };
    let response = (headers, data);
    Ok(response.into_response())
}

pub(crate) async fn generate_image(
    Path(name): Path<String>,
    Extension(args): Extension<Args>,
) -> Result<Response, AppError> {
    let path = StdPath::new(name.as_str());
    let name_extraction_error = error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Could not extract name from path",
    );
    match path.extension() {
        Some(ext) if ext == "png" => {
            let name = path.file_stem().and_then(|s| s.to_str());
            match name {
                Some(name) => generate_image_response(args, ImageType::Png, name),
                None => Ok(name_extraction_error),
            }
        }
        Some(ext) if ext == "jpg" || ext == "jpeg" => {
            let name = path.file_stem().and_then(|s| s.to_str());
            match name {
                Some(name) => generate_image_response(args, ImageType::Jpeg, name),
                None => Ok(name_extraction_error),
            }
        }
        _ => generate_image_response(args, ImageType::Png, name.as_str()),
    }
}
