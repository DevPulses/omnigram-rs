//! Desktop notification scheduling, queue limits, and read-clearing logic.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/window/notifications_manager.{h,cpp}`.

use models_history::flags::MsgId;
use models_peers::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerType {
    Dummy,
    Default,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationView {
    ShowAll,
    ShowNameOnly,
    ShowNothing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationSettings {
    pub desktop_enabled: bool,
    pub sound_enabled: bool,
    pub flash_bounce_enabled: bool,
    pub include_muted: bool,
    pub count_messages: bool,
    pub max_count: usize,
    pub view: NotificationView,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            desktop_enabled: true,
            sound_enabled: true,
            flash_bounce_enabled: true,
            include_muted: false,
            count_messages: true,
            max_count: 3,
            view: NotificationView::ShowAll,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationItem {
    pub session_id: u64,
    pub peer_id: PeerId,
    pub topic_id: MsgId,
    pub msg_id: MsgId,
    pub title: String,
    pub subtitle: String,
    pub body: String,
    pub is_muted: bool,
    pub is_mention: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedNotification {
    pub peer_id: PeerId,
    pub msg_id: MsgId,
    pub title: String,
    pub body: String,
    pub play_sound: bool,
}

#[derive(Debug, Default)]
pub struct NotificationSystem {
    pub manager_type: Option<ManagerType>,
    pub settings: NotificationSettings,
    active: Vec<NotificationItem>,
}

impl NotificationSystem {
    pub fn new(manager_type: ManagerType, settings: NotificationSettings) -> Self {
        Self {
            manager_type: Some(manager_type),
            settings,
            active: Vec::new(),
        }
    }

    pub fn schedule(&mut self, item: NotificationItem) -> Option<RenderedNotification> {
        if !self.settings.desktop_enabled {
            return None;
        }
        if item.is_muted && !item.is_mention && !self.settings.include_muted {
            return None;
        }

        // Deduplicate by (peer_id, msg_id)
        if self
            .active
            .iter()
            .any(|n| n.peer_id == item.peer_id && n.msg_id == item.msg_id)
        {
            return None;
        }

        if self.active.len() >= self.settings.max_count && !self.active.is_empty() {
            self.active.remove(0);
        }

        let rendered = self.format_item(&item);
        self.active.push(item);
        Some(rendered)
    }

    fn format_item(&self, item: &NotificationItem) -> RenderedNotification {
        let (title, body) = match self.settings.view {
            NotificationView::ShowAll => (item.title.clone(), item.body.clone()),
            NotificationView::ShowNameOnly => (item.title.clone(), "New message".to_string()),
            NotificationView::ShowNothing => ("Telegram".to_string(), "You have a new message".to_string()),
        };

        RenderedNotification {
            peer_id: item.peer_id,
            msg_id: item.msg_id,
            title,
            body,
            play_sound: self.settings.sound_enabled && !item.is_muted,
        }
    }

    pub fn clear_from_history(&mut self, peer_id: PeerId) {
        self.active.retain(|n| n.peer_id != peer_id);
    }

    pub fn clear_read(&mut self, peer_id: PeerId, read_till_id: MsgId) {
        self.active
            .retain(|n| n.peer_id != peer_id || n.msg_id > read_till_id);
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::UserId;

    fn notif(peer: u64, msg_id: MsgId, muted: bool) -> NotificationItem {
        NotificationItem {
            session_id: 1,
            peer_id: PeerId::from_user(UserId::new(peer)),
            topic_id: 0,
            msg_id,
            title: format!("User {peer}"),
            subtitle: String::new(),
            body: format!("Hello #{msg_id}"),
            is_muted: muted,
            is_mention: false,
        }
    }

    #[test]
    fn test_schedule_muted_privacy_and_clear_read() {
        let settings = NotificationSettings {
            view: NotificationView::ShowNameOnly,
            max_count: 2,
            ..Default::default()
        };
        let mut sys = NotificationSystem::new(ManagerType::Native, settings);

        // Muted chat skipped when include_muted is false
        assert!(sys.schedule(notif(1, 10, true)).is_none());

        let r1 = sys.schedule(notif(2, 11, false)).unwrap();
        assert_eq!(r1.title, "User 2");
        assert_eq!(r1.body, "New message");
        assert!(r1.play_sound);

        sys.schedule(notif(2, 12, false)).unwrap();
        // Exceeding max_count evicts oldest
        sys.schedule(notif(3, 13, false)).unwrap();
        assert_eq!(sys.active_count(), 2);

        sys.clear_read(PeerId::from_user(UserId::new(2)), 12);
        assert_eq!(sys.active_count(), 1);
    }
}
