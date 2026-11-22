// ── ALIEN Creative App Playbooks ── Phase G: Hardening ───────────────────────
//
// Specialized playbooks for high-productivity creative software:
//  - Clip Studio Paint
//  - Adobe Photoshop
//  - Blender 3D
//  - Unreal Engine 5
//  - MS Paint & Notepad
//
// Automatically initializes and loads playbooks from the Obsidian vault,
// injecting essential keyboard shortcuts, navigation quirks, and executive commands.

use crate::services::obsidian::ObsidianVault;

#[derive(Debug, Clone)]
pub struct AppPlaybook {
    pub app_name: &'static str,
    pub match_tokens: &'static [&'static str],
    pub content: &'static str,
}

pub static CREATIVE_PLAYBOOKS: &[AppPlaybook] = &[
    AppPlaybook {
        app_name: "Clip Studio Paint",
        match_tokens: &["clip studio", "clipstudiopaint", "clipstudio"],
        content: r#"## App Playbook: Clip Studio Paint
### Essential Shortcuts (Always Prefer Shortcuts over Menus)
- **Tools**:
  - `B`: Brush / Pen tool
  - `P`: Pen / Pencil tool
  - `E`: Eraser tool
  - `M`: Selection (Rectangle / Lasso)
  - `G`: Gradient / Fill Bucket
  - `I`: Eyedropper (Color Pick)
  - `[`: Decrease brush size
  - `]`: Increase brush size
  - `X`: Swap primary and secondary drawing colors
  - `C`: Toggle transparent color (erases using active brush texture!)
- **Navigation & Canvas**:
  - `Space + Drag`: Pan canvas smoothly
  - `Ctrl + +` / `Ctrl + -`: Zoom in / Zoom out
  - `Ctrl + 0`: Fit canvas to screen
- **Layers & Actions**:
  - `Ctrl + Z`: Undo | `Ctrl + Y`: Redo
  - `Ctrl + T`: Free Transform
  - `Ctrl + D`: Deselect active selection
  - `Ctrl + Shift + N`: New Raster Layer dialog
  - `Ctrl + S`: Save illustration (.clip)
### Critical UI Quirks
- Canvas viewport is GPU-accelerated and custom-drawn without child HWNDs.
- Rely on single-key shortcuts (`B`, `E`, `G`, `[`, `]`) rather than hunting through palette icons.
- When drawing or sketching, apply smooth drags across the canvas area."#,
    },
    AppPlaybook {
        app_name: "Adobe Photoshop",
        match_tokens: &["photoshop"],
        content: r#"## App Playbook: Adobe Photoshop
### Essential Shortcuts (Always Prefer Shortcuts over Menus)
- **Tools**:
  - `V`: Move tool
  - `M`: Marquee selection tool
  - `L`: Lasso selection tool
  - `B`: Brush tool
  - `E`: Eraser tool
  - `G`: Paint Bucket / Gradient tool
  - `I`: Eyedropper tool
  - `[`: Decrease brush size | `]`: Increase brush size
  - `Shift + [`: Softer brush edge | `Shift + ]`: Harder brush edge
  - `D`: Reset default foreground/background colors (Black & White)
  - `X`: Swap foreground and background colors
- **Navigation & Canvas**:
  - `Space + Drag`: Hand tool (Pan canvas)
  - `Ctrl + +` / `Ctrl + -`: Zoom in / Zoom out
  - `Ctrl + 0`: Fit on Screen
- **Layers & Actions**:
  - `Ctrl + Z`: Undo | `Ctrl + Shift + Z`: Redo
  - `Ctrl + T`: Free Transform
  - `Ctrl + J`: Duplicate Layer or Selection
  - `Ctrl + D`: Deselect
  - `Ctrl + Shift + N`: New Layer
  - `Ctrl + S`: Save | `Ctrl + Shift + Alt + W`: Export As
### Critical UI Quirks
- Sub-tools are deeply nested. NEVER hunt through tool flyouts; type the single-letter shortcut directly (`B`, `E`, `V`, `G`).
- Standard dialog boxes (File Save, Export, Filters) use standard Win32 controls; observe UI Automation hints for dialog buttons."#,
    },
    AppPlaybook {
        app_name: "Blender 3D",
        match_tokens: &["blender"],
        content: r#"## App Playbook: Blender 3D
### Essential Shortcuts (Always Prefer Shortcuts over Menus)
- **3D Viewport Navigation**:
  - `Middle Mouse Drag`: Orbit 3D viewport
  - `Shift + Middle Mouse Drag`: Pan 3D viewport
  - `Ctrl + Middle Mouse Drag` or `Scroll Wheel`: Zoom 3D viewport
  - `Numpad 1`: Front view | `Numpad 3`: Right view | `Numpad 7`: Top view | `Numpad 0`: Camera view
  - `Numpad .`: Frame selected object
- **Transformations (Modal Operations)**:
  - `G`: Grab / Move (Follow with `X`, `Y`, or `Z` to lock axis, click or Enter to confirm)
  - `R`: Rotate (Follow with `X`, `Y`, or `Z` to lock axis, click or Enter to confirm)
  - `S`: Scale (Follow with `X`, `Y`, or `Z` to lock axis, click or Enter to confirm)
  - `Esc` or Right Click: Cancel active transformation
- **Modeling & Modes**:
  - `Tab`: Toggle between Object Mode and Edit Mode
  - `A`: Select All | `Alt + A`: Deselect All
  - `Shift + A`: Add Menu (Mesh, Light, Camera, Curve)
  - `X` or `Delete`: Delete selected element
- **Executive Command Search**:
  - `F3`: Operator / Command Search (Instant access to any Blender operator without menu hunting!)
- **History**:
  - `Ctrl + Z`: Undo | `Ctrl + Shift + Z`: Redo
### Critical UI Quirks
- The entire Blender interface is rendered via custom OpenGL/Vulkan shaders.
- Do NOT attempt to find buttons in deeply nested header menus; press `F3`, type the exact tool name (e.g., "Subdivision Surface", "Extrude", "Bevel"), and hit Enter!"#,
    },
    AppPlaybook {
        app_name: "Unreal Engine 5",
        match_tokens: &["unreal", "unrealeditor"],
        content: r#"## App Playbook: Unreal Engine 5
### Essential Shortcuts (Always Prefer Shortcuts over Menus)
- **Viewport Navigation**:
  - `Hold Right Mouse Button + W/A/S/D/Q/E`: WASD flight navigation in 3D scene
  - `F`: Focus viewport camera on selected Actor
  - `G`: Toggle Game Mode (Hides editor icons and gizmos)
- **Transform Gizmos**:
  - `W`: Select and Translate mode
  - `E`: Select and Rotate mode
  - `R`: Select and Scale mode
  - `Ctrl + \`: Toggle between World and Local coordinate systems
- **Actor Operations**:
  - `Alt + Drag Gizmo`: Duplicate Actor
  - `Ctrl + D`: Duplicate selected Actor
  - `Delete`: Remove selected Actor
- **Drawers & Panes**:
  - `Ctrl + Space`: Toggle Content Drawer (Assets, Blueprints, Meshes)
  - `~` (Tilde): Toggle Output Log / Console
  - `Alt + P`: Play in Editor (PIE) | `Esc`: Stop simulation
  - `Ctrl + Shift + S`: Save All dirty packages
### Critical UI Quirks
- Slate engine draws custom widgets. Use `Ctrl + Space` to summon assets rather than manually clicking through directory trees."#,
    },
    AppPlaybook {
        app_name: "MS Paint",
        match_tokens: &["paint", "mspaint"],
        content: r#"## App Playbook: MS Paint
### Essential Shortcuts
- `B`: Brush tool
- `E` or `Ctrl + Alt + E`: Eraser tool
- `K`: Fill bucket tool
- `I`: Color picker (Eyedropper)
- `Ctrl + +`: Increase brush / line thickness
- `Ctrl + -`: Decrease brush / line thickness
- `Ctrl + S`: Save image"#,
    },
    AppPlaybook {
        app_name: "Notepad",
        match_tokens: &["notepad"],
        content: r#"## App Playbook: Notepad
### Essential Shortcuts
- `Ctrl + N`: New tab / document
- `Ctrl + S`: Save file
- `Ctrl + F`: Find text
- `Ctrl + A`: Select all
- `Ctrl + Z`: Undo"#,
    },
];

pub struct PlaybookManager;

impl PlaybookManager {
    /// Ensure all default creative playbooks are saved into the user's Obsidian vault.
    pub fn ensure_playbooks_initialized(vault: &ObsidianVault) {
        for pb in CREATIVE_PLAYBOOKS {
            let note_title = format!("App Playbook - {}", pb.app_name);
            // Check if note already exists, if not write it
            if vault.read_note("Instructions", &note_title).is_err() {
                let _ = vault.write_note("Instructions", &note_title, pb.content);
            }
        }
    }

    /// Match the active window title or user prompt against known playbooks.
    pub fn get_playbook_for_context(
        active_title: &str,
        user_prompt: &str,
        vault: &ObsidianVault,
    ) -> Option<String> {
        let title_lower = active_title.to_lowercase();
        let prompt_lower = user_prompt.to_lowercase();

        for pb in CREATIVE_PLAYBOOKS {
            let matches = pb.match_tokens.iter().any(|tok| {
                title_lower.contains(tok) || prompt_lower.contains(tok)
            });

            if matches {
                let note_title = format!("App Playbook - {}", pb.app_name);
                // Check if user has custom edits in their Obsidian vault
                if let Ok(custom_note) = vault.read_note("Instructions", &note_title) {
                    return Some(custom_note);
                } else {
                    return Some(pb.content.to_string());
                }
            }
        }

        None
    }
}
