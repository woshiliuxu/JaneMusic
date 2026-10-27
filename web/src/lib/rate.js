/* 防拉黑（限流保护）：60 秒窗口内自动重试最多 5 次，打满冷却一分钟。
   跳歌的指数退避放在 player 里，这里只管计数。 */
const RATE = { t0: 0, n: 0, coolUntil: 0 };
const RATE_WINDOW = 60000;
const RATE_MAX = 5;

export function rateReset() { RATE.t0 = 0; RATE.n = 0; }
export function rateClearCool() { RATE.coolUntil = 0; }
export function rateAllow() {
  const now = Date.now();
  if (now < RATE.coolUntil) return false;
  if (!RATE.t0 || now - RATE.t0 > RATE_WINDOW) { RATE.t0 = now; RATE.n = 0; }
  return RATE.n < RATE_MAX;
}
export function rateHit() {
  const now = Date.now();
  if (!RATE.t0 || now - RATE.t0 > RATE_WINDOW) { RATE.t0 = now; RATE.n = 0; }
  RATE.n++;
  if (RATE.n >= RATE_MAX) RATE.coolUntil = now + RATE_WINDOW;
  return RATE.n;
}
