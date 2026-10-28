// src/main.rs
mod agent;
mod tui;
mod orchestration;
mod cli;

use std::error::Error;
use crossterm::event::{self, Event};
use ratatui::Frame;

use tui::model::App;
use tui::{controller, view};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();          // 无论成功失败都先 restore 再返回的"三明治结构"
    result
}

// ratatui 0.30 的类型别名：Terminal<CrosstermBackend<Stdout>>。
// 旧版（<0.28）写作 ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>，
// 0.28 起官方提供 DefaultTerminal 别名，手写全称反而在 0.30 编译不过（E0425）。
fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame: &mut Frame| view::draw(&app, frame))?;
        if let Event::Key(key) = event::read()? {
            let cmd = controller::bind(&app, &key);
            controller::apply(&mut app, cmd);
        }
        if app.quit { break; }
    }
    Ok(())
}
