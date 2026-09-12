use tao::dpi::LogicalSize;
use tao::event_loop::{EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Window, WindowBuilder};
use wry::{http::Request, WebView, WebViewBuilder};

use crate::event::UserEvent;

const POPUP_HTML: &str = r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
  body { font-family: -apple-system, sans-serif; margin: 0; padding: 16px;
         background: #1e1e1e; color: #eee; }
  p { margin: 0 0 8px; font-size: 13px; }
  input { width: 100%; box-sizing: border-box; padding: 8px; font-size: 14px;
          border-radius: 4px; border: 1px solid #555; background: #2a2a2a; color: #eee; }
  button { margin-top: 10px; padding: 6px 16px; border-radius: 4px; border: none;
           background: #3a7afe; color: #fff; cursor: pointer; }
  #error { color: #ff6b6b; margin-top: 8px; min-height: 1.2em; font-size: 12px; }
</style>
</head>
<body>
  <p>移動先のURLを入力してください</p>
  <input id="url" type="text" placeholder="https://example.com" />
  <button id="go">開く</button>
  <button id="cancel" style="background:#555">キャンセル</button>
  <div id="error"></div>
  <script>
    function submit() {
      var value = document.getElementById('url').value;
      window.ipc.postMessage('navigate:' + value);
    }
    function cancel() {
      window.ipc.postMessage('cancel');
    }
    document.getElementById('go').addEventListener('click', submit);
    document.getElementById('cancel').addEventListener('click', cancel);
    document.getElementById('url').focus();
    document.getElementById('url').addEventListener('keydown', function (e) {
      if (e.key === 'Enter') submit();
      if (e.key === 'Escape') cancel();
    });
    function showError(msg) {
      document.getElementById('error').textContent = msg;
    }
  </script>
</body>
</html>
"#;

/// 文字列をJSの文字列リテラルとして安全に埋め込めるようエスケープする。
/// エラーメッセージにユーザー入力（URL文字列）がそのまま含まれ得るため、
/// `evaluate_script` へ渡す前に必ずこれを通す。
pub fn js_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '<' => out.push_str("\\u003C"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// URL手打ち用のポップアップウィンドウを新規作成する。
///
/// 戻る/進む機能を持たせない代わりに、任意のURLへ移動する唯一の手段として
/// このポップアップを使う。IPCで受け取った生の入力はここでは検証せず、
/// `event::UserEvent::Navigate` としてイベントループへ送るだけにする
/// （検証は `url_input::normalize_and_validate` に一本化する）。
pub fn create_url_popup(
    event_loop: &EventLoopWindowTarget<UserEvent>,
    proxy: EventLoopProxy<UserEvent>,
) -> wry::Result<(Window, WebView)> {
    let window = WindowBuilder::new()
        .with_title("URLを開く")
        .with_inner_size(LogicalSize::new(420.0, 160.0))
        .with_resizable(false)
        .build(event_loop)
        .expect("URL入力ポップアップの作成に失敗しました");

    let webview = WebViewBuilder::new()
        .with_html(POPUP_HTML)
        .with_ipc_handler(move |req: Request<String>| {
            let body = req.body().as_str();
            if let Some(raw) = body.strip_prefix("navigate:") {
                let _ = proxy.send_event(UserEvent::Navigate(raw.to_string()));
            } else if body == "cancel" {
                let _ = proxy.send_event(UserEvent::CloseUrlPopup);
            }
        })
        .build(&window)?;

    Ok((window, webview))
}
