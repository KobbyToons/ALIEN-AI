/**
 * Hook placeholder for global-shortcut management from the frontend.
 *
 * The actual Ctrl+Space shortcut is registered in Rust (lib.rs) and works
 * even when the window is hidden.  This hook exists so future phases can:
 *   - Let the user rebind the shortcut from Settings
 *   - Display the current binding in the UI
 */
export function useGlobalShortcut() {
  // Current binding — read-only for now.
  // Phase 3 (Settings UI) will make this configurable and call
  // the Rust backend to re-register.
  const binding = "Ctrl+Space";

  return { binding } as const;
}

