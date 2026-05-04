/**
 * Tiny global toast — single message at a time, auto-dismisses.
 * Used for transient confirmations (screenshot saved, copied to clipboard, …).
 */
class ToastStore {
  message = $state<string | null>(null);
  private timer: ReturnType<typeof setTimeout> | null = null;

  show(message: string, durationMs = 1400): void {
    this.message = message;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      this.message = null;
      this.timer = null;
    }, durationMs);
  }
}

export const toast = new ToastStore();
