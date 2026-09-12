use tao::event_loop::EventLoopProxy;
use tao::window::{Window, WindowBuilder};
use wry::{http::Request, NewWindowResponse, WebView, WebViewBuilder};

use crate::event::UserEvent;

const HOME_URL: &str = "https://www.google.com";

/// 全ページ・全フレームで実行される初期化スクリプト。
///
/// - `history.back/forward/go` をページ自身（サブフレーム含む）が呼び出しても
///   戻る/進むが起きないよう無効化する（戻る/進む機能を一切持たせないという
///   要件のため。`pushState`/`replaceState` はSPAの通常動作なので許可する）。
///   iframeの `history` は全フレーム共通のセッション履歴を操作できるため、
///   メインフレームだけでなく全フレームに適用する必要がある。
/// - 右上に固定の「URLを開く」ボタンを注入する（サイト側のCSSに影響されない
///   よう最大z-indexのインラインstyleで配置）。ボタンはメインフレームにのみ
///   表示する。
const INIT_SCRIPT: &str = r#"
(function () {
  history.back = function () {};
  history.forward = function () {};
  history.go = function () {};

  if (window.top !== window) return;

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
    document.body.appendChild(btn);
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
    let new_window_proxy = proxy.clone();

    WebViewBuilder::new()
        .with_url(HOME_URL)
        // 全フレームに適用しないと、iframe経由の history.back() 呼び出しを
        // 防げない（joint session historyが動いてしまう）。
        .with_initialization_script_for_main_only(INIT_SCRIPT, false)
        // スワイプ等による戻る/進むジェスチャーを無効化する。
        .with_back_forward_navigation_gestures(false)
        .with_navigation_handler(is_navigation_allowed)
        // target="_blank" や window.open() による新規ウィンドウ作成は拒否しつつ、
        // 要求されたURLは同じWebView内で遷移させる（別ウィンドウ/タブを増やさず、
        // かつリンクを無反応にしない）。
        .with_new_window_req_handler(move |url, _features| {
            let _ = new_window_proxy.send_event(UserEvent::Navigate(url));
            NewWindowResponse::Deny
        })
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
        // ネイティブのフルスクリーン（Spaces切り替えを伴う）ではなく、通常の
        // ウィンドウのまま画面いっぱいに広げる「最大化」で起動する。
        .with_maximized(true)
        .build(event_loop)
        .expect("メインウィンドウの作成に失敗しました")
}
