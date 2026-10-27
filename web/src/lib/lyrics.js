/* 歌词：LRC 解析 + 按时间定位当前句（纯函数） */

export function parseLrc(text) {
  const lines = [];
  if (!text) return lines;
  for (const raw of text.split(/\r?\n/)) {
    const m = raw.match(/^\s*\[(\d+):(\d+)(?:\.(\d+))?\](.*)$/);
    if (!m) continue;
    const t = +m[1] * 60 + +m[2] + (m[3] ? +("0." + m[3]) : 0);
    const txt = m[4].trim();
    if (txt) lines.push({ t, txt });
  }
  lines.sort((a, b) => a.t - b.t);
  return lines;
}

/* 当前播放时间对应的歌词行下标（-1 = 还没唱到第一句） */
export function indexAt(lines, cur) {
  let idx = -1;
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].t <= cur) idx = i;
    else break;
  }
  return idx;
}

export function fmt(sec) {
  if (!isFinite(sec) || sec < 0) sec = 0;
  const m = Math.floor(sec / 60), s = Math.floor(sec % 60);
  return m + ":" + String(s).padStart(2, "0");
}
