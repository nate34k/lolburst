use crossterm::event::KeyCode;
use tui_logger::TuiWidgetEvent;

use crate::app::{App, UIEvent};

pub enum KeyboardHandler {
    Quit,
    None,
}

pub fn handle_keyboard(ui_event: UIEvent, app: &mut App) -> KeyboardHandler {
    match ui_event {
        UIEvent::Key(key_event) => {
            match key_event.code {
                KeyCode::Char('q') => {
                    return KeyboardHandler::Quit;
                }
                KeyCode::Char('l') => {
                    info!("Toggling logger on/off");
                    app.draw_logger = !app.draw_logger;
                }
                KeyCode::PageUp => {
                    app.logger_state.transition(&TuiWidgetEvent::PrevPageKey);
                    app.logger_scroll_mode = true;
                }
                KeyCode::PageDown => {
                    app.logger_state.transition(&TuiWidgetEvent::NextPageKey);
                    app.logger_scroll_mode = true;
                }
                KeyCode::Up => {
                    app.logger_state.transition(&TuiWidgetEvent::UpKey);
                }
                KeyCode::Down => {
                    app.logger_state.transition(&TuiWidgetEvent::DownKey);
                }
                KeyCode::Left => {
                    app.logger_state.transition(&TuiWidgetEvent::LeftKey);
                }
                KeyCode::Right => {
                    app.logger_state.transition(&TuiWidgetEvent::RightKey);
                }
                KeyCode::Esc => {
                    app.logger_state.transition(&TuiWidgetEvent::EscapeKey);
                    app.logger_scroll_mode = false;
                }
                KeyCode::Char(' ') => {
                    app.logger_state.transition(&TuiWidgetEvent::SpaceKey);
                }
                KeyCode::Char('+') => {
                    app.logger_state.transition(&TuiWidgetEvent::PlusKey);
                }
                KeyCode::Char('-') => {
                    app.logger_state.transition(&TuiWidgetEvent::MinusKey);
                }
                KeyCode::Char('h') => {
                    app.logger_state.transition(&TuiWidgetEvent::HideKey);
                }
                KeyCode::Char('f') => {
                    app.logger_state.transition(&TuiWidgetEvent::FocusKey);
                }
                _ => {}
            }
            debug!("{:?}", key_event);
        }
        UIEvent::Resize(_x, _y) => {}
    }
    KeyboardHandler::None
}
