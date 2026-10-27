/* 版本检查：启动时探一次 GitHub 最新 Release，有新版就亮标题栏的 NEW。
   走 WebView2 自身网络（和用户浏览器同一条路），探不到就安静待着，绝不打扰。 */

import { store, toast } from "./store.js";
import { invoke } from "./tauri.js";
import { APP_VERSION, REPO_URL } from "./constants.js";

const API_LATEST = "https://api.github.com/repos/woshiliuxu/JaneMusic/releases/latest";
const CHECK_INTERVAL = 6 * 3600 * 1000; // 之后每 6 小时补探一次

/* "v4.0.1" vs "4.0.2" → 1 表示远端更新 */
function compareVer(local, remote) {
  const p = (s) => String(s).replace(/^v/, "").split(".").map(Number);
  const a = p(local), b = p(remote);
  for (let i = 0; i < 3; i++) {
    if ((b[i] || 0) > (a[i] || 0)) return 1;
    if ((b[i] || 0) < (a[i] || 0)) return -1;
  }
  return 0;
}

export async function checkUpdate(manual = false) {
  try {
    const r = await fetch(API_LATEST, {
      headers: { Accept: "application/vnd.github+json" },
      cache: "no-store",
    });
    if (!r.ok) throw new Error("http " + r.status);
    const j = await r.json();
    const latest = j.tag_name || "";
    if (latest && compareVer(APP_VERSION, latest) > 0) {
      /* 找 Release 里的 exe 资产：有就能走「下载替换重启」的一键更新，没有只能开下载页 */
      const exeAsset = (j.assets || []).find((a) => /\.exe$/i.test(a.name || ""));
      store.update = {
        latest,
        url: j.html_url || REPO_URL + "/releases/latest",
        exe: exeAsset ? exeAsset.browser_download_url : "",
      };
      if (!manual) toast(`出了新版本 ${latest}，标题栏亮了颗 NEW，点它去拿`, "ok");
      return true;
    }
    if (manual) toast("不用更，你这就是最新的", "ok");
    return false;
  } catch {
    if (manual) toast("没探到，多半是网络不给面子", "err");
    return false;
  }
}

/* 一键更新：下载新 exe → 替换 → 自动重启（全在 Rust 侧做，这里只管提示与兜底）。
   成功时进程会被重启、走不到后面；任何失败都退回「打开下载页自己下」。 */
export async function applyUpdate() {
  const u = store.update;
  if (!u || store.updating) return;
  if (!u.exe) {
    /* Release 里没挂 exe（或还是老 Release）：直接开下载页 */
    invoke("open_url", { url: u.url }).catch(() => {});
    return;
  }
  store.updating = true;
  toast(`正在下载 ${u.latest}，好了会自动重启`, "ok");
  try {
    await invoke("update_app", { url: u.exe });
    /* 正常情况下 Rust 已替换并退出进程，下面只是兜底 */
    store.updating = false;
  } catch (e) {
    store.updating = false;
    toast("自动更新没成，去下载页手动拿吧", "err");
    invoke("open_url", { url: u.url }).catch(() => {});
  }
}

export function scheduleUpdateCheck() {  /* 错开启动时的榜单请求，8 秒后再探；探完按间隔补探 */
  setTimeout(() => {
    checkUpdate(false);
    setInterval(() => checkUpdate(false), CHECK_INTERVAL);
  }, 8000);
}
