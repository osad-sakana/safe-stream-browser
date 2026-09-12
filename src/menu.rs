use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tao::event_loop::EventLoopProxy;

use crate::event::UserEvent;

/// macOSの画面上部メニューバーを構築し、「URLを開く」メニュー項目の選択を
/// `UserEvent::OpenUrlPopup` としてイベントループへ転送する。
///
/// 戻る/進むボタンを持たせない代わりに、任意のURLへ移動する手段は
/// このメニュー（と、そこから開くポップアップ）に一本化する。
///
/// 戻り値の `Menu` は呼び出し元（`main`）でイベントループの寿命が尽きるまで
/// 保持すること。`Menu` は最後の参照がドロップされた時点でネイティブの
/// メニューを破棄する実装になっており、ここで返さずに関数内で終わらせると
/// 生成直後にメニューバーが消えてしまう。
pub fn setup_menu(proxy: EventLoopProxy<UserEvent>) -> Menu {
    let menu = Menu::new();

    let app_menu = Submenu::new("SafeStream Browser", true);
    let _ = app_menu.append(&PredefinedMenuItem::quit(Some("SafeStream Browserを終了")));
    let _ = menu.append(&app_menu);

    let navigate_menu = Submenu::new("移動", true);
    let open_url_item = MenuItem::new(
        "URLを開く...",
        true,
        Some(Accelerator::new(Modifiers::META, Code::KeyL)),
    );
    let _ = navigate_menu.append(&open_url_item);
    let _ = menu.append(&navigate_menu);

    #[cfg(target_os = "macos")]
    menu.init_for_nsapp();

    let open_url_id = open_url_item.id().clone();
    muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        if event.id == open_url_id {
            let _ = proxy.send_event(UserEvent::OpenUrlPopup);
        }
    }));

    menu
}
