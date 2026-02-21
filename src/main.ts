import { createApp } from "vue";
import App from "./App.vue";
import router from "@/router";
import "@/styles/main.css";

function installMobileOverscrollGuard() {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return;
  }
  const isTouchDevice =
    "ontouchstart" in window || navigator.maxTouchPoints > 0;
  if (!isTouchDevice) {
    return;
  }

  let touchStartY = 0;

  const findScrollableAncestor = (start: EventTarget | null): HTMLElement | null => {
    let node = start instanceof HTMLElement ? start : null;
    while (node && node !== document.body) {
      const style = window.getComputedStyle(node);
      const overflowY = style.overflowY;
      const scrollableY =
        (overflowY === "auto" || overflowY === "scroll") &&
        node.scrollHeight > node.clientHeight;
      if (scrollableY) {
        return node;
      }
      node = node.parentElement;
    }
    return null;
  };

  document.addEventListener(
    "touchstart",
    (event) => {
      touchStartY = event.touches[0]?.clientY ?? 0;
    },
    { passive: true },
  );

  document.addEventListener(
    "touchmove",
    (event) => {
      const currentY = event.touches[0]?.clientY ?? touchStartY;
      const deltaY = currentY - touchStartY;
      const container = findScrollableAncestor(event.target);

      if (!container) {
        event.preventDefault();
        return;
      }

      const atTop = container.scrollTop <= 0;
      const atBottom =
        container.scrollTop + container.clientHeight >= container.scrollHeight - 1;

      if ((atTop && deltaY > 0) || (atBottom && deltaY < 0)) {
        event.preventDefault();
      }
    },
    { passive: false },
  );
}

installMobileOverscrollGuard();

createApp(App).use(router).mount("#app");
