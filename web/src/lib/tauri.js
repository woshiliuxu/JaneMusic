/* Tauri 桥：invoke、本地流式代理（真 HTTP 支持 Range，绕开 WebView2 网络栈） */

export const invoke = (cmd, args) => window.__TAURI_INTERNALS__.invoke(cmd, args);

let streamPort = 0;
export function setStreamPort(p) { streamPort = p || 0; }
export function getStreamPort() { return streamPort; }

export function base(u, alt) {
  return streamPort
    ? "http://127.0.0.1:" + streamPort + "/?u=" + encodeURIComponent(u) +
      (alt ? "&alt=" + encodeURIComponent(alt) : "")
    : "";
}
export const proxied = (u) => base(u, "");
/* 音频带上备用源：主源给不出音频时后端会自己换一个再试 */
export const proxiedSong = (s) => base(s.url, s.alt || "");
