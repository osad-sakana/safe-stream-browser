use tao::event_loop::EventLoopProxy;
use tao::window::{Window, WindowBuilder};
use wry::{http::Request, NewWindowResponse, WebView, WebViewBuilder};

use crate::event::UserEvent;

const HOME_URL: &str = "https://www.google.com";

/// 全ページで実行される初期化スクリプト。
///
/// - 右上に固定の「URLを開く」ボタンを注入する（サイト側のCSSに影響されない
///   よう最大z-indexのインラインstyleで配置）。
/// - `history.back/forward/go` をページ自身が呼び出しても戻る/進むが
///   起きないよう無効化する（戻る/進む機能を一切持たせないという要件のため。
///   `pushState`/`replaceState` はSPAの通常動作なので許可する）。
const INIT_SCRIPT: &str = r#"
(function () {
  history.back = function () {};
  history.forward = function () {};
  history.go = function () {};

  function injectOverlay() {
    if (document.getElementById('__safe_stream_open_url_btn__')) return;
    var btn = document.createElement('button');
    btn.id = '__safe_stream_open_url_btn__';
    btn.textContent = 'URLを開く';
    btn.style.cssText = [
      'position:fixed', 'top:8px', 'right:8px', 'z-index:2147483647',
      'padding:6px 12px', 'font-size:12px', 'font-family:sans-serif',
      'background:#222', 'color:#fff', 'border:1px solid #555',
      'border-radius:6px', 'cursor:pointer', 'opacity:0.85'
    ].join(';');
    btn.addEventListener('click', function () {
      window.ipc.postMessage('open-url-popup');
    });
    document.documentElement.appendChild(btn);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', injectOverlay);
  } else {
    injectOverlay();
  }
})();
"#;

/// リンククリック等によるナビゲーション先を http/https のみに制限する。
/// `file:` や `javascript:` への遷移（配信者のローカルファイル露出や
/// スクリプト注入につながる）を防ぐ。
fn is_navigation_allowed(url: String) -> bool {
    url::Url::parse(&url)
        .map(|u| matches!(u.scheme(), "http" | "https"))
        .unwrap_or(false)
}

pub fn create_main_webview(
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
) -> wry::Result<WebView> {
    WebViewBuilder::new()
        .with_url(HOME_URL)
        .with_initialization_script(INIT_SCRIPT)
        // スワイプ等による戻る/進むジェスチャーを無効化する。
        .with_back_forward_navigation_gestures(false)
        .with_navigation_handler(is_navigation_allowed)
        // window.open() 等による新規ウィンドウ作成を拒否し、意図しないタブ/
        // ウィンドウが増えないようにする。
        .with_new_window_req_handler(|_url, _features| NewWindowResponse::Deny)
        .with_ipc_handler(move |req: Request<String>| {
            if req.body() == "open-url-popup" {
                let _ = proxy.send_event(UserEvent::OpenUrlPopup);
            }
        })
        .build(window)
}

pub fn create_main_window(
    event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
) -> Window {
    WindowBuilder::new()
        .with_title("SafeStream Browser")
        .build(event_loop)
        .expect("メインウィンドウの作成に失敗しました")
}
