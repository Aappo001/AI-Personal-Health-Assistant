use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use printpdf::{PdfDocument, Mm};
use std::fs::File;
use crate::AppState;
use crate::auth::JwtAuth;
use crate::users::UserToken;
use crate::error::AppError;
use std::io::BufWriter;
use crate::forms::HealthForm;

pub async fn generate_pdf_report(
    State(state): State<AppState>,
    JwtAuth(user): JwtAuth<UserToken>,
) -> Result<Response, AppError> {
    // Fetch all forms for the user
    let data = sqlx::query_as!(
        HealthForm,
        "SELECT * FROM user_statistics WHERE user_id = ?",
        user.id
    )
    .fetch_all(&state.pool)
    .await?;
    
    // Calculate averages
    let total_entries = data.len() as f64;
    let sleep_hours_avg = data.iter().filter_map(|f| f.sleep_hours).sum::<f64>() / total_entries;
    let exercise_duration_avg = data.iter().filter_map(|f| f.exercise_duration).sum::<f64>() / total_entries;

    // Create a PDF document
    let (doc, page1, layer1) = PdfDocument::new("User Health Report", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load external font
    let font = doc.add_external_font(File::open("path/to/Helvetica.ttf")?)?;

    // Add content to the PDF
    current_layer.use_text(
        format!("Health Statistics Report for User ID: {}", user.id),
        24.0,
        Mm(10.0),
        Mm(280.0),
        &font,
    );
    current_layer.use_text(
        format!("Average Sleep Hours: {:.2}", sleep_hours_avg),
        16.0,
        Mm(10.0),
        Mm(250.0),
        &font,
    );
    current_layer.use_text(
        format!("Average Exercise Duration: {:.2} minutes", exercise_duration_avg),
        16.0,
        Mm(10.0),
        Mm(230.0),
        &font,
    );

    // Save to a buffer
    let mut buffer = Vec::new();
    doc.save(&mut BufWriter::new(&mut buffer))?;

    // Return PDF response
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"health_report.pdf\""),
        ],
        buffer,
    ).into_response())
}
