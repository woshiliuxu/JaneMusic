/* 死链跨会话记忆：只存后端验明 404/410/415 的歌（jm.dead.v2，换 key 自动抛弃旧污染） */
const DEAD_KEY = "jm.dead.v2";
const DEAD_CAP = 800;

function load() {
  try {
    const arr = JSON.parse(localStorage.getItem(DEAD_KEY) || "[]");
    return new Set(Array.isArray(arr) ? arr : []);
  } catch { return new Set(); }
}

const dead = load();

export const keyOf = (u) => u.slice(-80);
export function isDead(url) { return dead.has(keyOf(url)); }
export function markDead(url) {
  dead.add(keyOf(url));
  try {
    const arr = [...dead].slice(-DEAD_CAP);
    localStorage.setItem(DEAD_KEY, JSON.stringify(arr));
  } catch { /* 存不下就算了 */ }
}
