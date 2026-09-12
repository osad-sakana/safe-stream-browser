mod browser;
mod event;
mod url_input;
mod url_popup;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use wry::WebView;

use event::UserEvent;

fn main() -> wry::Result<()> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let main_window = browser::create_main_window(&event_loop);
    let main_window_id = main_window.id();
    let main_webview = browser::create_main_webview(&main_window, proxy.clone())?;

    let mut popup: Option<(tao::window::Window, WebView)> = None;

    event_loop.run(move |event, event_loop_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                if window_id == main_window_id {
                    *control_flow = ControlFlow::Exit;
                } else if popup.as_ref().is_some_and(|(w, _)| w.id() == window_id) {
                    popup = None;
                }
            }
            Event::UserEvent(UserEvent::OpenUrlPopup) => {
                if let Some((window, _)) = &popup {
                    window.set_focus();
                } else {
                    match url_popup::create_url_popup(event_loop_target, proxy.clone()) {
                        Ok(pair) => popup = Some(pair),
                        Err(err) => eprintln!("URL入力ポップアップの作成に失敗しました: {err}"),
                    }
                }
            }
            Event::UserEvent(UserEvent::CloseUrlPopup) => {
                popup = None;
            }
            Event::UserEvent(UserEvent::Navigate(raw)) => match url_input::normalize_and_validate(&raw) {
                Ok(url) => {
                    if let Err(err) = main_webview.load_url(&url) {
                        eprintln!("URLへの遷移に失敗しました: {err}");
                    }
                    popup = None;
                }
                Err(err) => {
                    if let Some((_, webview)) = &popup {
                        let script = format!(
                            "showError({})",
                            url_popup::js_string_literal(&err.to_string())
                        );
                        let _ = webview.evaluate_script(&script);
                    }
                }
            },
            _ => {}
        }
    });
}
