#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 默认启动参数：
/// - `--in-process-gpu`：本机 WebView2 独立 GPU 进程必崩（黑屏/不出窗口），GPU 必须跑在进程内；
/// - `--disable-features=AudioServiceOutOfProcess`：本机音频服务（utility 进程 + 沙箱）起不来，
///   表现为数据全缓冲好了、PLAYING 也触发了，但 currentTime 永远是 0，随后
///   err3 "AUDIO_RENDERER_ERROR: audio render error"。把音频服务收回浏览器进程即可出声。
///
/// 调参：设环境变量 JM_WEBVIEW_ARGS 可整体覆盖（排查问题时免重编译）。
const DEFAULT_ARGS: &str = "--in-process-gpu --disable-features=AudioServiceOutOfProcess";

fn main() {
    let args = std::env::var("JM_WEBVIEW_ARGS").unwrap_or_else(|_| DEFAULT_ARGS.to_string());
    std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args);
    jane_music_lib::run()
}
