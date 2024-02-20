use std::io::Read;

use shuten_core::{
    event::{Event, Key, MouseEvent},
    geom::{lerp, Pos2, Rect},
    style::Rgb,
    terminal::Config,
    Canvas, Cell, Terminal,
};

fn main() -> std::io::Result<()> {
    let lines = match std::env::args().nth(1).as_deref() {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let mut terminal = Terminal::new(Config::default())?;
    let mut app = App::new(lines.lines());

    // initial paint
    terminal.paint(|mut canvas| app.draw_ui(&mut canvas))?;

    while let Ok(event) = terminal.wait_for_next_event() {
        match event {
            Event::Mouse(ev, _) => {
                if let MouseEvent::Scroll { dir, .. } = ev {
                    if dir.y.is_negative() {
                        app.scroll_down(3)
                    } else {
                        app.scroll_up(3)
                    }
                }
                app.cursor = ev.pos()
            }

            Event::Keyboard(Key::Char(' '), ..) => app.enabled = !app.enabled,

            Event::Keyboard(Key::PageUp, ..) => {
                app.scroll_down(terminal.rect().height() as usize * 2)
            }

            Event::Keyboard(Key::PageDown, ..) => {
                app.scroll_up(terminal.rect().height() as usize * 2)
            }

            Event::Keyboard(Key::Up, ..) => app.scroll_down(1),
            Event::Keyboard(Key::Down, ..) => app.scroll_up(1),

            Event::Quit => break,
            _ => continue,
        }

        terminal.paint(|mut canvas| app.draw_ui(&mut canvas))?;
    }

    Ok(())
}

struct App {
    cursor: Pos2,
    enabled: bool,
    lines: Vec<String>,
    pos: usize,
}

impl App {
    fn new<S>(lines: impl IntoIterator<Item = S>) -> Self
    where
        S: ToString,
    {
        let lines = lines.into_iter().map(|s| s.to_string()).collect::<Vec<_>>();
        Self {
            cursor: Pos2::default(),
            enabled: false,
            pos: lines.len(),
            lines,
        }
    }

    fn scroll_up(&mut self, delta: usize) {
        self.pos = self.pos.saturating_sub(delta);
    }

    fn scroll_down(&mut self, delta: usize) {
        self.pos = (self.pos + delta).min(self.lines.len());
    }
}

impl App {
    const FG: Rgb = Rgb::from_u32(0x000000);
    const BG: Rgb = Rgb::from_u32(0xF0E68C);
    const SHADOW: Rgb = Rgb::from_u32(0x333333);

    fn draw_ui(&self, canvas: &mut Canvas) {
        canvas.fill(if self.enabled { Self::FG } else { Self::BG });

        let rect = canvas.area();
        let offset = self.lines.len().saturating_sub(self.pos);
        let offset = offset
            .checked_sub(rect.height().saturating_sub(1) as usize)
            .unwrap_or(offset);

        let width = rect.width();
        let mut start = rect.left_top();
        for line in self.lines.iter().skip(offset) {
            if start.y >= rect.height() {
                break;
            }

            for c in line.chars() {
                if start.x >= width {
                    start.x = rect.left();
                    start.y += 1;
                }
                canvas.put(start, self.maybe_blend(start, c));
                start.x += 1;
            }

            // fill in the rest of the line
            while start.x < rect.width() {
                canvas.put(start, self.maybe_blend(start, ' '));
                start.x += 1;
            }
            start.x = rect.left();
            start.y += 1;
        }

        // fill in the rest of the screen
        if start.y < rect.height() {
            for pos in Rect::from_min_max(start, rect.max).indices() {
                canvas.put(pos, self.maybe_blend(pos, ' '));
            }
        }
    }

    fn maybe_blend(&self, pos: Pos2, c: char) -> Cell {
        if !self.enabled {
            return Cell::new(c).fg(Self::FG).bg(Self::BG);
        }

        // length
        let x = pos.x as f32 - self.cursor.x as f32;
        let y = pos.y as f32 - self.cursor.y as f32;

        // fix the aspect ratio (probably wrong for not-my-setup)
        let x = x * 1.6;
        let y = y * 3.0;

        let distance = x.hypot(y).sqrt().max(1.5);
        let blend = lerp(0.0..=0.25, distance);

        Cell::new(c)
            .fg(Self::FG)
            .bg(Self::BG.blend_flat(Self::SHADOW, blend))
    }
}
