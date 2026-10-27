/* 界面级动作：主题、置顶（跨视图共用） */
import { store, kv, kvSave, toast } from "./store.js";
import { invoke } from "./tauri.js";
import { paintTint } from "./tint.js";

export function applyTheme(mode) {
  store.theme = mode;
  document.documentElement.setAttribute("data-appearance", mode);
  localStorage.setItem("jm.theme", mode);
  kv.theme = mode;
  kvSave();
}

export function toggleTheme() {
  applyTheme(store.theme === "dark" ? "light" : "dark");
  paintTint(store.theme); // 明暗变了，光晕亮度跟着重算
}

export async function togglePin() {
  store.pinned = await invoke("toggle_pin").catch(() => store.pinned);
  toast(store.pinned ? "窗口钉住了，跑不掉" : "松开手，窗口自由了", "ok");
}

/* 歌词字号：写 CSS 变量（三处歌词共用）+ 落盘 */
export function applyLyrSize(px) {
  const v = Math.max(12, Math.min(24, Math.round(+px || 15)));
  store.lyrSize = v;
  kv.lyrSize = v;
  document.documentElement.style.setProperty("--lyr-base", v + "px");
  localStorage.setItem("jm.lyrSize", String(v));
  kvSave();
  return v;
}
