//! Status filters and viewport state survive refreshes of the shared task projection.

mod hints;
mod input;
mod navigation;
mod render;
mod rows;

use super::*;

// Counts and filtering use the same status groups.
pub(super) const TASK_FILTERS: &[(&str, Option<AgentsOverviewGroup>)] = &[
    #[cfg(feature = "custom-agents-overview")]
    ("My agents", None),
    #[cfg(not(feature = "custom-agents-overview"))]
    ("All", None),
    ("Needs you", Some(AgentsOverviewGroup::NeedsYou)),
    ("Working", Some(AgentsOverviewGroup::Working)),
    ("Ready", Some(AgentsOverviewGroup::Ready)),
    #[cfg(not(feature = "custom-agents-overview"))]
    ("Inactive", Some(AgentsOverviewGroup::Finished)),
    #[cfg(feature = "custom-agents-overview")]
    ("All", None),
];

impl AgentsOverviewView {
    pub(in crate::app::agents_overview_view) fn can_show_more(&self) -> bool {
        let state = self.state();
        #[cfg(feature = "custom-agents-overview")]
        if state.status_filter == 0 {
            return false;
        }
        state.has_more
    }

    pub(in crate::app::agents_overview_view) fn selectable_indices(&self) -> Vec<usize> {
        let mut indices = self.visible_indices();
        if self.can_show_more() {
            indices.push(usize::MAX);
        }
        indices
    }

    pub(in crate::app::agents_overview_view) fn reconcile_command_center_selection(&mut self) {
        let visible = self.selectable_indices();
        if !visible.contains(&self.selected) {
            self.selected = visible.first().copied().unwrap_or(usize::MAX);
            self.state().show_more_selected = false;
        }
    }
}
