/* 音量：淡入淡出 ramp + 手调（手调永远赢过淡变） */
import { audio, store, kv, kvSave } from "./store.js";

let rampTimer = null;
export function cancelRamp() {
  if (rampTimer) { clearInterval(rampTimer); rampTimer = null; }
}

export function rampTo(pct, dur, done) {
  cancelRamp();
  const from = audio.volume;
  const to = Math.max(0, Math.min(1, pct / 100));
  if (Math.abs(to - from) < 0.005) { audio.volume = to; if (done) done(); return; }
  const t0 = performance.now();
  rampTimer = setInterval(() => {
    const k = Math.min(1, (performance.now() - t0) / dur);
    audio.volume = from + (to - from) * k;
    if (k >= 1) {
      clearInterval(rampTimer);
      rampTimer = null;
      if (done) done();
    }
  }, 16);
}

/* 手调音量：立即生效并落盘 */
export function applyVol(v) {
  cancelRamp();
  v = Math.max(0, Math.min(100, Math.round(v)));
  store.vol = v;
  audio.volume = v / 100;
  audio.muted = v === 0;
  saveVol(v, true);
}

export function saveVol(v, skipApply) {
  if (!skipApply) { /* applyVol 已带 */ }
  localStorage.setItem("jm.vol", String(v));
  kv.vol = v;
  kvSave();
}
