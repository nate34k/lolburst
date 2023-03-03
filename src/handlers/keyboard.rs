use crossterm::event::KeyCode;
use tui_logger::TuiWidgetEvent;

use crate::app::{App, UIEvent};

#[derive(Debug, PartialEq)]
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
                    app.logger_scroll_freeze = true;
                }
                KeyCode::PageDown => {
                    app.logger_state.transition(&TuiWidgetEvent::NextPageKey);
                    app.logger_scroll_freeze = true;
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
                    app.logger_scroll_freeze = false;
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

#[test]
fn test_handle_keyboard_q_press() {
    let mut app = App::new(&crate::config::Config::default());
    let ui_event = UIEvent::Key(crossterm::event::KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    let handler = handle_keyboard(ui_event, &mut app);
    assert_eq!(handler, KeyboardHandler::Quit);
}

#[test]
fn test_handle_keyboard_l_press() {
    let mut app = App::new(&crate::config::Config::default());
    let ui_event = UIEvent::Key(crossterm::event::KeyEvent {
        code: KeyCode::Char('l'),
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    let handler = handle_keyboard(ui_event, &mut app);
    assert_eq!(handler, KeyboardHandler::None);
    assert_eq!(app.draw_logger, true);
}

#[test]
fn test_handle_keyboard_page_up_press() {
    let mut app = App::new(&crate::config::Config::default());
    let ui_event = UIEvent::Key(crossterm::event::KeyEvent {
        code: KeyCode::PageUp,
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    let handler = handle_keyboard(ui_event, &mut app);
    assert_eq!(handler, KeyboardHandler::None);
    assert_eq!(app.logger_scroll_freeze, true);
}

#[test]
fn test_handle_keyboard_page_down_press() {
    let mut app = App::new(&crate::config::Config::default());
    let ui_event = UIEvent::Key(crossterm::event::KeyEvent {
        code: KeyCode::PageDown,
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    let handler = handle_keyboard(ui_event, &mut app);
    assert_eq!(handler, KeyboardHandler::None);
    assert_eq!(app.logger_scroll_freeze, true);
}

#[test]
fn test_handle_keyboard_esc_press() {
    let mut app = App::new(&crate::config::Config::default());
    let ui_event = UIEvent::Key(crossterm::event::KeyEvent {
        code: KeyCode::Esc,
        modifiers: crossterm::event::KeyModifiers::NONE,
    });
    let handler = handle_keyboard(ui_event, &mut app);
    assert_eq!(handler, KeyboardHandler::None);
    assert_eq!(app.logger_scroll_freeze, false);
}


