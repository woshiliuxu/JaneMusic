import { createApp } from "vue";
import App from "./App.vue";
import "./styles/global.css";

/* 桌面应用不带浏览器右键菜单；输入框里保留原生菜单（复制/粘贴）。 */
window.addEventListener("contextmenu", (e) => {
  const t = e.target;
  const editable =
    t instanceof HTMLElement &&
    (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable);
  if (!editable) e.preventDefault();
});

createApp(App).mount("#app");
