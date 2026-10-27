/* 封面取色：24x24 加权主色 → 一冷一暖两团环境光（跟随明暗主题调亮度） */

const tint = { h: null, s: 0 };

function rgbToHsl(r, g, b) {
  r /= 255; g /= 255; b /= 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b);
  const l = (mx + mn) / 2;
  if (mx === mn) return [0, 0];
  const d = mx - mn;
  const s = l > 0.5 ? d / (2 - mx - mn) : d / (mx + mn);
  let h;
  if (mx === r) h = (g - b) / d + (g < b ? 6 : 0);
  else if (mx === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;
  return [h * 60, s];
}

export function paintTint(theme) {
  if (tint.h === null) return;
  const dark = theme === "dark";
  const s = Math.min(60, Math.max(30, tint.s * 100));
  const a1 = "hsla(" + tint.h.toFixed(0) + ", " + s.toFixed(0) + "%, " + (dark ? 26 : 76) + "%, " + (dark ? .40 : .44) + ")";
  const a2 = "hsla(" + ((tint.h + 26) % 360).toFixed(0) + ", " + (s * .85).toFixed(0) + "%, " + (dark ? 30 : 83) + "%, " + (dark ? .30 : .34) + ")";
  const st = document.documentElement.style;
  st.setProperty("--amb1", a1);
  st.setProperty("--amb2", a2);
}

export function pickTint(img, theme) {
  try {
    const N = 24;
    const cv = document.createElement("canvas");
    cv.width = cv.height = N;
    const ctx = cv.getContext("2d", { willReadFrequently: true });
    ctx.drawImage(img, 0, 0, N, N);
    const d = ctx.getImageData(0, 0, N, N).data;
    let r = 0, g = 0, b = 0, w = 0;
    for (let i = 0; i < d.length; i += 4) {
      if (d[i + 3] < 180) continue;
      const R = d[i], G = d[i + 1], B = d[i + 2];
      const mx = Math.max(R, G, B), mn = Math.min(R, G, B);
      const lum = (mx + mn) / 2 / 255;
      if (lum > 0.93 || lum < 0.07) continue;
      const sat = mx === mn ? 0 : (mx - mn) / (255 - Math.abs(mx + mn - 255));
      const weight = 0.3 + sat * sat * 1.8;
      r += R * weight; g += G * weight; b += B * weight; w += weight;
    }
    if (w === 0) return;
    const [h, s] = rgbToHsl(r / w, g / w, b / w);
    tint.h = h; tint.s = s;
    paintTint(theme);
  } catch { /* canvas 被跨域污染就保持默认背景 */ }
}
