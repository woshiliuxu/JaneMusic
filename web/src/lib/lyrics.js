/* 歌词：LRC 解析 + 按时间定位当前句（纯函数） */

/* 外语歌的现成写法是「一句原文后面跟中文翻译」：`미안해 하지마 (不要说对不起)`。
   整句塞一行又长又挤，这里把尾部括号里的翻译摘出来（sub），交给界面换行、小一号显示。
   判定条件缺一不可：
     - 括号里得有汉字（翻译）；`(Live)`、`(Oh yeah)` 这种不拆
     - 括号外得有外语字母（拉丁 / 谚文 / 假名 / 西里尔 / 变音拉丁）；
       纯中文歌词里的旁白，如「走吧（笑）」，括号外也是中文，一律不拆 */
const RE_CJK = /[\u3400-\u9fff]/;
const RE_FOREIGN = /[A-Za-z\u00c0-\u024f\u0370-\u04ff\uac00-\ud7af\u3040-\u30ff]/;

function splitBilingual(txt) {
  const m = txt.match(/^(.*?)\s*[（(]\s*([^（）()]*?)\s*[)）]\s*$/);
  if (!m) return { txt, sub: "" };
  const main = m[1].trim();
  const inner = m[2].trim();
  if (!main || !inner) return { txt, sub: "" };
  if (!RE_CJK.test(inner)) return { txt, sub: "" };
  if (!RE_FOREIGN.test(main)) return { txt, sub: "" };
  return { txt: main, sub: inner };
}

export function parseLrc(text) {
  const lines = [];
  if (!text) return lines;
  for (const raw of text.split(/\r?\n/)) {
    const m = raw.match(/^\s*\[(\d+):(\d+)(?:\.(\d+))?\](.*)$/);
    if (!m) continue;
    const t = +m[1] * 60 + +m[2] + (m[3] ? +("0." + m[3]) : 0);
    const txt = m[4].trim();
    if (txt) lines.push({ t, ...splitBilingual(txt) });
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
