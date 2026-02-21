import { inject, type InjectionKey } from "vue";
import { useGameApp } from "@/composables/useGameApp";

export type GameAppContext = ReturnType<typeof useGameApp>;

export const gameAppContextKey: InjectionKey<GameAppContext> = Symbol("game-app-context");

export function useGameAppContext(): GameAppContext {
  const context = inject(gameAppContextKey);
  if (!context) {
    throw new Error("useGameAppContext must be used within App provider");
  }
  return context;
}
