use color_eyre::eyre::Result;
use colored::Colorize;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    symbols::Marker,
    text::Line,
    widgets::{
        Block,
        BorderType::Thick,
        Paragraph, Widget,
        canvas::{Canvas, Rectangle},
    },
};
use std::time::{Duration, Instant};

enum Facing {
    Up,
    Down,
    Left,
    Right,
}

struct Point {
    x: f64,
    y: f64,
}

struct AppState {
    snake: Vec<Point>,
    dir: Facing,

    food: Point,
    score: u8,
    clock: u8,

    freq: u64,

    width: f64,
    height: f64,

    status: bool,
}

impl AppState {
    fn update(&mut self) {
        // Remove last value, add new head.
        let head = &self.snake[0];
        let mut new_head = Point {
            x: head.x,
            y: head.y,
        };

        match self.dir {
            Facing::Down => new_head.y -= 1.0,
            Facing::Up => new_head.y += 1.0,
            Facing::Left => new_head.x -= 1.0,
            Facing::Right => new_head.x += 1.0,
        }

        if new_head.x >= self.width
            || new_head.x < 0.0
            || new_head.y >= self.height
            || new_head.y < 0.0
        {
            self.end_game();
        }

        if self
            .snake
            .iter()
            .any(|seg| seg.x == new_head.x && seg.y == new_head.y)
        {
            self.end_game();
        }

        if new_head.x == self.food.x && new_head.y == self.food.y {
            self.snake.insert(0, new_head);
            self.score += 1;
            self.add_food();
        } else {
            self.snake.insert(0, new_head);
            self.snake.pop();
        }
    }

    fn change_dir(&mut self, new_dir: Facing) {
        match (&self.dir, &new_dir) {
            (Facing::Down, Facing::Up) | (Facing::Up, Facing::Down) => {}
            (Facing::Right, Facing::Left) | (Facing::Left, Facing::Right) => {}
            _ => {
                self.dir = new_dir;
            }
        }
    }

    fn add_food(&mut self) {
        while self
            .snake
            .iter()
            .any(|seg| seg.x == self.food.x && seg.y == self.food.y)
        {
            self.food.x = rand::random_range(0..(self.width as u8)) as f64;
            self.food.y = rand::random_range(0..(self.height as u8)) as f64;
        }

        if self.freq > 70 {
            self.freq -= 3;
        }
    }

    fn end_game(&mut self) {
        self.status = true;
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let state = &mut AppState {
        snake: vec![
            Point { x: 9.0, y: 21.0 }, // <- HEAD
            Point { x: 8.0, y: 21.0 },
            Point { x: 7.0, y: 21.0 },
            Point { x: 6.0, y: 21.0 },
            Point { x: 5.0, y: 21.0 }, // <- TAIL
        ],
        dir: Facing::Right,
        food: Point { x: 21.0, y: 21.0 },
        score: 0,
        clock: 0,

        freq: 180,

        width: 40.0,
        height: 40.0,

        status: false,
    };

    let terminal = ratatui::init();
    let result = run(terminal, state);

    ratatui::restore();

    println!(
        "Game finished with {}! Play again by running {}.",
        Colorize::bold(format!("{} points", state.score).as_str()).yellow(),
        Colorize::bold("snake").blue()
    );

    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    let tick_rate = Duration::from_millis(app_state.freq); // Controls game speed (lower = faster)
    let mut last_tick = Instant::now();

    loop {
        // Behaviour
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Renderer
        terminal.draw(|f| render(f, app_state))?;

        // Input handler
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    event::KeyCode::Esc => {
                        break;
                    }

                    event::KeyCode::Left => {
                        app_state.change_dir(Facing::Left);
                    }
                    event::KeyCode::Up => {
                        app_state.change_dir(Facing::Up);
                    }
                    event::KeyCode::Down => {
                        app_state.change_dir(Facing::Down);
                    }
                    event::KeyCode::Right => {
                        app_state.change_dir(Facing::Right);
                    }

                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app_state.update();
            last_tick = Instant::now();
        }

        if app_state.status {
            break;
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(23),
            Constraint::Fill(1),
        ])
        .split(frame.area());

    let middle_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(63),
            Constraint::Length(15),
            Constraint::Fill(1),
        ])
        .split(main_area[1]);

    let center_box = Block::bordered().fg(Color::Gray).border_type(Thick);

    let info_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(middle_area[2]);

    let score_box = Block::bordered()
        .fg(Color::Yellow)
        .border_type(Thick)
        .title_top(Line::from("[ SCORE ]").centered().bold());

    let timer_box = Block::bordered()
        .fg(Color::Blue)
        .border_type(Thick)
        .title_top(Line::from("[ TIME ]").centered().bold());

    Paragraph::new(Line::from(format!("{}", app_state.score)).centered())
        .block(score_box)
        .render(info_area[0], frame.buffer_mut());

    // Paragraph::new(Line::from(format!("{}", app_state.score)).centered())
    //     .block(timer_box)
    //     .render(info_area[1], frame.buffer_mut());

    let game = Canvas::default()
        .block(center_box)
        .marker(Marker::Octant)
        .x_bounds([0.0, app_state.width])
        .y_bounds([0.0, app_state.height])
        .paint(|ctx| {
            for piece in &app_state.snake {
                ctx.draw(&Rectangle {
                    x: piece.x,
                    y: piece.y,
                    width: 1.0,
                    height: 1.0,
                    color: Color::Green,
                })
            }

            ctx.draw(&Rectangle {
                x: app_state.food.x,
                y: app_state.food.y,
                width: 1.0,
                height: 1.0,
                color: Color::Red,
            })
        });

    frame.render_widget(game, middle_area[1]);
}
