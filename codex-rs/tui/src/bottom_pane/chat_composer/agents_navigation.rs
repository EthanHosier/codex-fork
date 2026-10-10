//! Availability of the empty-prompt agents shortcut and its footer hint.
//!
//! Only local-daemon sessions enable this shortcut. Draft contents and transient input surfaces
//! take precedence; in the legacy path, editor remaps can also disable the Left shortcut.

use super::*;

impl ChatComposer {
    pub(crate) fn set_agents_navigation_enabled(&mut self, enabled: bool) {
        self.agents_navigation_enabled = enabled;
    }

    pub(crate) fn agents_navigation_key_available(&self) -> bool {
        #[cfg(feature = "custom-agents-overview")]
        {
            true
        }
        #[cfg(not(feature = "custom-agents-overview"))]
        {
            let move_left = if self.draft.textarea.is_vim_normal_mode() {
                &self.vim_normal_keymap.move_left
            } else {
                &self.editor_keymap.move_left
            };
            move_left.is_pressed(KeyCode::Left.into())
        }
    }

    pub(super) fn agents_navigation_available(&self) -> bool {
        self.agents_navigation_enabled
            && self.agents_navigation_key_available()
            && self.has_focus
            && self.draft.input_enabled
            && self.slash_commands_enabled()
            && self.is_empty()
            && !self.is_in_paste_burst()
            && !self.popup_active()
            && !self.draft.textarea.is_vim_operator_pending()
    }
}
