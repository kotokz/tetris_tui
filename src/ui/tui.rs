use std::{
    io,
    time::{Duration, Instant},
};

use crate::game::tetris::Tetris;
use crate::game::{shape::Shape, tetris::TetrisBoard};

use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::{error, info};
use tui_logger::TuiLoggerWidget;

#[derive(PartialEq, Eq)]
enum GameState {
    Quit,
    Failed,
}

pub fn run_tui_app() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    loop {
        let mut tetris = TetrisBoard::new_default();
        let state = run_game_loop(&mut terminal, &mut tetris)?;
        if state == GameState::Quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('n') => continue,
                _ => {}
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;
    Ok(())
}

fn run_game_loop<B: Backend, T: Tetris>(
    terminal: &mut Terminal<B>,
    tetris: &mut T,
) -> io::Result<GameState> {
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    info!("Game started!");
    loop {
        terminal.draw(|f| draw_game(f, tetris))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(GameState::Quit),
                    KeyCode::Up => tetris.rotate(),
                    KeyCode::Left => tetris.shift(crate::game::tetris::Direction::Left),
                    KeyCode::Right => tetris.shift(crate::game::tetris::Direction::Right),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            tetris.tick();
            last_tick = Instant::now();
        }

        if !tetris.alive() {
            return Ok(GameState::Failed);
        }
    }
}

fn draw_game<B: Backend, T: Tetris>(f: &mut Frame<B>, tetris: &mut T) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let title = draw_title();
    f.render_widget(title, chunks[0]);

    draw_game_board(f, tetris, chunks[1]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Tetris TUI")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

impl From<Shape> for Color {
    fn from(shape: Shape) -> Color {
        match shape {
            Shape::I => Color::Rgb(0, 255, 255),
            Shape::O => Color::Rgb(255, 255, 0),
            Shape::T => Color::Rgb(128, 0, 128),
            Shape::J => Color::Rgb(0, 255, 0),
            Shape::L => Color::Rgb(255, 0, 0),
            Shape::S => Color::Rgb(0, 0, 255),
            Shape::Z => Color::Rgb(255, 127, 0),
        }
    }
}

fn draw_game_board<B: Backend, T: Tetris>(f: &mut Frame<B>, tetris: &mut T, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .margin(2)
        .split(area);

    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Game Board",
        Style::default().fg(Color::Rgb(170, 143, 121)),
    ));

    f.render_widget(block, area);

    // Logs
    let logs = draw_logs();
    f.render_widget(logs, chunks[0]);

    let (width, height) = tetris.board_size();

    let area_len = std::cmp::min(chunks[1].width, chunks[1].height) as i32;

    if area_len < width || area_len < height {
        error!("The screen is too small to show game board");
        return;
    }

    let block_ratio = std::cmp::min(area_len / width, area_len / height);

    let block_width = block_ratio * width;
    let block_height = block_ratio * height;

    let block_area = Rect {
        x: chunks[1].x + 2,
        y: chunks[1].y,
        width: block_width as u16,
        height: block_height as u16,
    };

    let board_cells = split_rect_into_tetris_squre(block_area, width, height);

    for (index, cell) in board_cells.into_iter().enumerate() {
        let (x, y) = convert_index_to_cords(index as i32, width);
        let color: Color = if let Some(shape) = tetris.get((x, y).into()) {
            shape.into()
        } else {
            Color::Rgb(127, 127, 127)
        };

        let block = Block::default()
            .style(Style::default().bg(color))
            .border_type(BorderType::Plain)
            .borders(Borders::ALL);

        f.render_widget(block, cell);
    }
}

fn split_rect_into_tetris_squre(area: Rect, width: i32, height: i32) -> Vec<Rect> {
    let mut rets = vec![];

    let rows = split_rect_by_direction(area, height, Direction::Vertical);
    for row in rows {
        rets.extend(split_rect_by_direction(row, width, Direction::Horizontal));
    }
    rets
}
fn split_rect_by_direction(area: Rect, counts: i32, dir: Direction) -> Vec<Rect> {
    let constraints: Vec<Constraint> = (0..counts)
        .map(|_| Constraint::Ratio(1, counts as u32))
        .collect();

    Layout::default()
        .direction(dir)
        .constraints(constraints.as_ref())
        .margin(0)
        .split(area)
}

fn convert_index_to_cords(index: i32, width: i32) -> (i32, i32) {
    let x = index % width;
    let y = index / width;
    (x, y)
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(Block::default().title("Logs").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        assert_eq!(convert_index_to_cords(0, 10), (0, 0));
        assert_eq!(convert_index_to_cords(11, 10), (1, 1));
        assert_eq!(convert_index_to_cords(25, 10), (5, 2));
    }
}
