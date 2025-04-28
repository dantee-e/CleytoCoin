use std::sync::Arc;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    prelude::*
};
use crate::node::logger::Logger;

/// The main application which holds the state and logic of the application.

pub struct App {
    /// Is the application running?
    running: bool,
    port: u16,
    logger: Arc<Logger>
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(logger: Arc<Logger>, port: u16) -> Self {
        Self {
            running: true,
            port,
            logger
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(frame.area());

        let main_title = Line::from(" CleytoCoin node is running! ")
            .bold()
            .blue()
            .centered();
        let logs_title = Line::from(" LOGS ")
            .bold()
            .blue()
            .centered();
        let port = self.port;
        let text = format!("Node running in port {port}
            \n\nPress `Esc`, `Ctrl-C` or `q` to stop running.");


        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(main_title))
                .centered(),
            layout[0],
        );
        frame.render_widget(
            Paragraph::new(self.logger.read_temp_logs().unwrap().join("\n"))
                .block(Block::bordered().title(logs_title))
                .left_aligned(),
            layout[1],
        )
    }


    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }


    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }


    fn quit(&mut self) {
        self.running = false;
    }
}