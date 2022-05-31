use tui::{Terminal, backend::Backend, Frame};
use tokio::{sync::watch, task::spawn_blocking};

trait RenderState: Clone + Send {
    fn render(&self, f: &mut Frame<impl Backend>);
}

pub async fn start_render_loop(mut terminal: Terminal<impl Backend + Sync + Send + 'static>, mut recv: watch::Receiver<impl RenderState + 'static>) -> std::io::Result<()> {
    loop {
        if let Err(err) = recv.changed().await {
            println!("{err:#?}");
            break;
        }

        let state = recv.borrow().clone();
        // Maybe moving in and out is expensive, so boxing might be a better idea, but let's wait
        // and profile.
        terminal = spawn_blocking(move || {
            terminal.draw(|f| {
                state.render(f);
            });
            return terminal;
        }).await?;
    }

    Ok(())
}
