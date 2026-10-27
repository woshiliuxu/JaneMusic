/* 常量：榜单、尺寸、持久化键 */

export const CATS = [
  { name: "热歌榜", id: "3778678" },
  { name: "新歌榜", id: "3779629" },
  { name: "飙升榜", id: "19723756" },
  { name: "抖音热歌", id: "2250011882" },
  { name: "怀旧经典", id: "908655286" },
];

export const MAX_HIST = 120;
export const MAX_RETRY = 60;
export const PL_TTL = 3 * 24 * 3600 * 1000;

export const SIZES = {
  big: { w: 860, h: 600 },
  small: { w: 380, h: 600 },
  mini: { w: 340, h: 84 },
};

export const WHEEL_STEP = 60;
export const WHEEL_COOL = 320;

/* 版本与仓库（发版时与 src-tauri/tauri.conf.json 的 version 一起改） */
export const APP_VERSION = "4.0.2";
export const REPO_URL = "https://github.com/woshiliuxu/JaneMusic";
