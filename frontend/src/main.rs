use app::App;

mod app;
mod widgets;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let mut app = App::new().await?;
    app.start(&mut terminal).await?;

    ratatui::restore();
    Ok(())
}
