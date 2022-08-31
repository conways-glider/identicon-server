use std::path::Path as StdPath;

use axum::extract::Query;
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::{extract::Path, response::IntoResponse, Extension};
use identicon_rs::Identicon;
use serde::Deserialize;
use tracing::{error, info};

use crate::errors::{self, AppError};
use crate::AppData;
enum ImageType {
    Png,
    Jpeg,
}

#[derive(Deserialize)]
pub(crate) struct ImageQueryParams {
    size: Option<u32>,
    scale: Option<u32>,
    border: Option<u32>,
    mirrored: Option<bool>,
}

fn generate_image_response(
    args: AppData,
    params: ImageQueryParams,
    image_type: ImageType,
    input: String,
) -> Result<Response, AppError> {
    info!("Generating image data for {}", &input);
    // Create identicon
    let mut identicon = Identicon::new(&input);

    // Get the identicon parameters from the query parameters, args, or defaults
    let border = params.border.unwrap_or(args.border);
    let size = params.size.unwrap_or(args.size);
    let scale = params.scale.unwrap_or(args.scale);
    let mirrored = params.mirrored.unwrap_or(args.mirrored);

    // Configure the identicon parameters
    identicon.set_border(border);
    identicon.set_size(size).map_err(AppError::Identicon)?;
    identicon.set_scale(scale).map_err(AppError::Identicon)?;
    identicon.set_mirrored(mirrored);

    // Generate the identicon
    let data = match image_type {
        ImageType::Png => identicon.export_png_data().unwrap(),
        ImageType::Jpeg => identicon.export_jpeg_data().unwrap(),
    };

    // Determine header
    info!("Generating image headers for {}", input);
    let headers = match image_type {
        ImageType::Png => [(header::CONTENT_TYPE, "image/png".to_string())],
        ImageType::Jpeg => [(header::CONTENT_TYPE, "image/jpeg".to_string())],
    };

    // Return the response
    let response = (headers, data);
    Ok(response.into_response())
}

pub(crate) fn generate_image(
    name: String,
    params: ImageQueryParams,
    args: AppData,
) -> Result<Response, AppError> {
    info!("Generating image for {}", name);
    let path = StdPath::new(&name);
    let name_extraction_error = errors::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Could not extract name from path",
    );
    match path.extension() {
        Some(ext) if ext == "png" => {
            let name = path.file_stem().and_then(|s| s.to_str());
            match name {
                Some(name) => {
                    generate_image_response(args, params, ImageType::Png, name.to_string())
                }
                None => Ok(name_extraction_error),
            }
        }
        Some(ext) if ext == "jpg" || ext == "jpeg" => {
            let name = path.file_stem().and_then(|s| s.to_str());
            match name {
                Some(name) => {
                    generate_image_response(args, params, ImageType::Jpeg, name.to_string())
                }
                None => Ok(name_extraction_error),
            }
        }
        _ => generate_image_response(args, params, ImageType::Png, name),
    }
}

pub(crate) async fn generate_image_path(
    Path(name): Path<String>,
    Query(params): Query<ImageQueryParams>,
    Extension(args): Extension<AppData>,
) -> Result<Response, AppError> {
    if args.scale <= args.size {
        let err = errors::AppError::ScaleTooSmall {
            scale: args.scale,
            size: args.size,
        };
        error! {"{}", err};
        Err(err)
    } else if args.scale > 1024 {
        let err = errors::AppError::ScaleTooLarge(args.scale);
        error! {"{}", err};
        Err(err)
    } else {
        // Use rayon as identicon generation is a CPU bottleneck
        let (send, recv) = tokio::sync::oneshot::channel();

        // Spawn a task on rayon.
        rayon::spawn(move || {
            // Perform an expensive computation.
            let image_result = generate_image(name, params, args);
            // Send the result back to Tokio.
            let _ = send.send(image_result);
        });

        // Wait for the rayon task.
        recv.await.expect("Panic in rayon::spawn")
    }
}
