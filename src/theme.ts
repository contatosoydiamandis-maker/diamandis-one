export type ThemePreference = "dark" | "light" | "system";

let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((event: MediaQueryListEvent) => void) | null = null;

export function setThemePreference(theme: ThemePreference): void {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener("change", mediaListener);
  }

  mediaQuery = null;
  mediaListener = null;

  const apply = (isLight: boolean) => {
    document.documentElement.dataset.theme =
      theme === "system" ? (isLight ? "light" : "dark") : theme;
  };

  if (theme !== "system") {
    apply(false);
    return;
  }

  mediaQuery = window.matchMedia("(prefers-color-scheme: light)");
  apply(mediaQuery.matches);
  mediaListener = event => apply(event.matches);
  mediaQuery.addEventListener("change", mediaListener);
}
