mod url_input;

use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::{NewWindowResponse, WebViewBuilder};

const HOME_URL: &str = "https://www.google.com";

/// リンククリック等によるナビゲーション先を http/https のみに制限する。
/// `file:` や `javascript:` への遷移（配信者のローカルファイル露出や
/// スクリプト注入につながる）を防ぐ。
fn is_navigation_allowed(url: String) -> bool {
    url::Url::parse(&url)
        .map(|u| matches!(u.scheme(), "http" | "https"))
        .unwrap_or(false)
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("SafeStream Browser")
        .build(&event_loop)
        .expect("メインウィンドウの作成に失敗しました");

    let _webview = WebViewBuilder::new()
        .with_url(HOME_URL)
        // スワイプ等による戻る/進むジェスチャーを無効化する。
        .with_back_forward_navigation_gestures(false)
        .with_navigation_handler(is_navigation_allowed)
        // window.open() 等による新規ウィンドウ作成を拒否し、意図しないタブ/
        // ウィンドウが増えないようにする。
        .with_new_window_req_handler(|_url, _features| NewWindowResponse::Deny)
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
