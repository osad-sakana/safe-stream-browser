/// イベントループ内で扱うアプリケーション独自イベント。
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// URL手打ち用ポップアップウィンドウを開く（既に開いていれば前面に出す）。
    OpenUrlPopup,
    /// ポップアップウィンドウを閉じる。
    CloseUrlPopup,
    /// メインウィンドウのWebViewを指定URLへ遷移させる。
    /// 遷移前に呼び出し元で正規化・検証済みであることを前提とする。
    Navigate(String),
}
