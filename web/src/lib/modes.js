/* 三档窗口模式：大窗 760×600（默认）/ 小窗 380×600 / 胶囊 340×84。
   固定尺寸、不允许拖边缩放；胶囊只在托盘菜单进。
   切换顺序沿用 3.0 的零闪烁经验：先换布局（html[data-mode]）再改窗口尺寸，
   展开方向加 130ms 淡入盖过重绘；胶囊自动置顶，退出恢复原置顶状态。 */
import { store, kv, kvSave, closeModal } from "./store.js";
import { invoke } from "./tauri.js";
import { SIZES } from "./constants.js";

export async function setMode(mode) {
  if (mode === store.mode || store.modeBusy) return;
  if (mode !== "big" && mode !== "small" && mode !== "mini") return;
  store.modeBusy = true;

  if (mode === "mini") {
    ["settings", "hist", "lyr", "favs", "list"].forEach(closeModal);
    if (store.mode !== "mini") { store.prevWin = store.mode; kv.lastWin = store.mode; }
    kv.mode = "mini";
    kvSave();
    store.pinBeforeMini = await invoke("get_pin").catch(() => false);
    store.mode = "mini"; // html[data-mode] 翻转，布局树不拆、胶囊盖上来
    const s = SIZES.mini;
    await invoke("set_window_size", { w: s.w, h: s.h, rounded: false }).catch(() => {});
    invoke("set_pin", { on: true }).catch(() => {});
    store.pinned = true;
    store.modeBusy = false;
    return;
  }

  const fromMini = store.mode === "mini";
  store.mode = mode;
  kv.mode = mode;
  kv.lastWin = mode;
  kvSave();
  const s = SIZES[mode];
  await invoke("set_window_size", { w: s.w, h: s.h, rounded: true }).catch(() => {});
  if (fromMini) {
    document.documentElement.classList.add("app-in");
    setTimeout(() => document.documentElement.classList.remove("app-in"), 200);
    const back = !!store.pinBeforeMini;
    invoke("set_pin", { on: back }).catch(() => {});
    store.pinned = back;
  }
  store.modeBusy = false;
}
