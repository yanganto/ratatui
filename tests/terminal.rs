use std::error::Error;

use ratatui::{
    assert_buffer_eq,
    backend::{Backend, TestBackend},
    layout::Rect,
    prelude::Buffer,
    widgets::{Paragraph, Widget},
    Terminal, TerminalOptions, Viewport,
};

#[test]
fn terminal_buffer_size_should_be_limited() {
    let backend = TestBackend::new(400, 400);
    let terminal = Terminal::new(backend).unwrap();
    let size = terminal.backend().size().unwrap();
    assert_eq!(size.width, 255);
    assert_eq!(size.height, 255);
}

#[test]
fn swap_buffer_clears_prev_buffer() {
    let backend = TestBackend::new(100, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .current_buffer_mut()
        .set_string(0, 0, "Hello", ratatui::style::Style::reset());
    assert_eq!(terminal.current_buffer_mut().content()[0].symbol(), "H");
    terminal.swap_buffers();
    assert_eq!(terminal.current_buffer_mut().content()[0].symbol(), " ");
}

#[test]
fn terminal_draw_returns_the_completed_frame() -> Result<(), Box<dyn Error>> {
    let backend = TestBackend::new(10, 10);
    let mut terminal = Terminal::new(backend)?;
    let frame = terminal.draw(|f| {
        let paragraph = Paragraph::new("Test");
        f.render_widget(paragraph, f.size());
    })?;
    assert_eq!(frame.buffer.get(0, 0).symbol(), "T");
    assert_eq!(frame.area, Rect::new(0, 0, 10, 10));
    terminal.backend_mut().resize(8, 8);
    let frame = terminal.draw(|f| {
        let paragraph = Paragraph::new("test");
        f.render_widget(paragraph, f.size());
    })?;
    assert_eq!(frame.buffer.get(0, 0).symbol(), "t");
    assert_eq!(frame.area, Rect::new(0, 0, 8, 8));
    Ok(())
}

#[test]
fn terminal_insert_before_moves_viewport() -> Result<(), Box<dyn Error>> {
    // When we have a terminal with 5 lines, and a single line viewport, if we insert a
    // number of lines less than the `terminal height - viewport height` it should move
    // viewport down to accommodate the new lines.

    let backend = TestBackend::new(20, 5);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(1),
        },
    )?;

    // insert_before cannot guarantee the contents of the viewport remain unharmed
    // by potential scrolling as such it is necessary to call draw afterwards to
    // redraw the contents of the viewport over the newly designated area.
    terminal.insert_before(2, |buf| {
        Paragraph::new(vec![
            "------ Line 1 ------".into(),
            "------ Line 2 ------".into(),
        ])
        .render(buf.area, buf);
    })?;

    terminal.draw(|f| {
        let paragraph = Paragraph::new("[---- Viewport ----]");
        f.render_widget(paragraph, f.size());
    })?;

    assert_buffer_eq!(
        terminal.backend().buffer().clone(),
        Buffer::with_lines(vec![
            "------ Line 1 ------",
            "------ Line 2 ------",
            "[---- Viewport ----]",
            "                    ",
            "                    ",
        ])
    );

    Ok(())
}

#[test]
fn terminal_insert_before_scrolls_on_large_input() -> Result<(), Box<dyn Error>> {
    // When we have a terminal with 5 lines, and a single line viewport, if we insert many
    // lines before the viewport (greater than `terminal height - viewport height`) it should
    // move the viewport down to the bottom of the terminal and scroll all lines above the viewport
    // until all have been added to the buffer.

    let backend = TestBackend::new(20, 5);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(1),
        },
    )?;

    terminal.insert_before(5, |buf| {
        Paragraph::new(vec![
            "------ Line 1 ------".into(),
            "------ Line 2 ------".into(),
            "------ Line 3 ------".into(),
            "------ Line 4 ------".into(),
            "------ Line 5 ------".into(),
        ])
        .render(buf.area, buf);
    })?;

    terminal.draw(|f| {
        let paragraph = Paragraph::new("[---- Viewport ----]");
        f.render_widget(paragraph, f.size());
    })?;

    assert_buffer_eq!(
        terminal.backend().buffer().clone(),
        Buffer::with_lines(vec![
            "------ Line 2 ------",
            "------ Line 3 ------",
            "------ Line 4 ------",
            "------ Line 5 ------",
            "[---- Viewport ----]",
        ])
    );

    Ok(())
}

#[test]
fn terminal_insert_before_scrolls_on_many_inserts() -> Result<(), Box<dyn Error>> {
    // This test ensures similar behaviour to `terminal_insert_before_scrolls_on_large_input`
    // but covers a bug previously present whereby multiple small insertions
    // (less than `terminal height - viewport height`) would have disparate behaviour to one large
    // insertion. This was caused by an undocumented cap on the height to be inserted, which has now
    // been removed.

    let backend = TestBackend::new(20, 5);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(1),
        },
    )?;

    terminal.insert_before(1, |buf| {
        Paragraph::new(vec!["------ Line 1 ------".into()]).render(buf.area, buf);
    })?;

    terminal.insert_before(1, |buf| {
        Paragraph::new(vec!["------ Line 2 ------".into()]).render(buf.area, buf);
    })?;

    terminal.insert_before(1, |buf| {
        Paragraph::new(vec!["------ Line 3 ------".into()]).render(buf.area, buf);
    })?;

    terminal.insert_before(1, |buf| {
        Paragraph::new(vec!["------ Line 4 ------".into()]).render(buf.area, buf);
    })?;

    terminal.insert_before(1, |buf| {
        Paragraph::new(vec!["------ Line 5 ------".into()]).render(buf.area, buf);
    })?;

    terminal.draw(|f| {
        let paragraph = Paragraph::new("[---- Viewport ----]");
        f.render_widget(paragraph, f.size());
    })?;

    assert_buffer_eq!(
        terminal.backend().buffer().clone(),
        Buffer::with_lines(vec![
            "------ Line 2 ------",
            "------ Line 3 ------",
            "------ Line 4 ------",
            "------ Line 5 ------",
            "[---- Viewport ----]",
        ])
    );

    Ok(())
}
