struct State {
}

impl RenderState for State {
    fn draw_ui(f: &mut Frame<impl Backend>, state: &RenderState) {
        let chunks = Layout::default()
            .direction(Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.size());

        let child_window = Block::default()
            .title("stdout")
            .borders(Borders::ALL)
            .border_type(BorderType::Double);

        f.render_widget(child_window, chunks[0]);

        let cmd_window = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        let cmd_position = cmd_window.inner(chunks[1]);
        f.set_cursor(cmd_position.left() + UnicodeWidthStr::width(state.command_text.as_str()) as u16, cmd_position.top());
        let cmd_text = Paragraph::new(state.command_text.as_str()).block(cmd_window);
        f.render_widget(cmd_text, chunks[1])
    }
}
