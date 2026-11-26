use crate::element::ElementGeometry;
use data_media_types::Size;
use models_history::item::HistoryItem;
use ui_message_bubble::MessageBubbleItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDeliveryStatus {
    Sending,
    Sent,
    Read,
}

#[derive(Debug, Clone)]
pub struct MessageView {
    pub id: i32,
    pub is_out: bool,
    pub delivery_status: MessageDeliveryStatus,
    pub date_text: String,
    pub bubble_item: MessageBubbleItem,
    pub geometry: ElementGeometry,
    pub media_size: Option<Size>,
}

impl MessageView {
    pub fn new(item: &HistoryItem, date_str: &str, text_w: i32, text_h: i32, container_w: i32) -> Self {
        let is_out = item.is_outgoing();
        let bubble_item = MessageBubbleItem::new(item.id, item.from, item.text.clone(), is_out);
        let bubble_geom = bubble_item.count_geometry(text_w, text_h, container_w);

        Self {
            id: item.id,
            is_out,
            delivery_status: if is_out {
                if item.is_unread() {
                    MessageDeliveryStatus::Sent
                } else {
                    MessageDeliveryStatus::Read
                }
            } else {
                MessageDeliveryStatus::Read
            },
            date_text: date_str.to_string(),
            bubble_item,
            geometry: ElementGeometry::new(bubble_geom.x, bubble_geom.y, bubble_geom.width, bubble_geom.height),
            media_size: None,
        }
    }

    pub fn set_media_size(&mut self, size: Size, text_w: i32, text_h: i32, container_w: i32) {
        self.media_size = Some(size);
        let base_geom = self.bubble_item.count_geometry(text_w.max(size.width), text_h, container_w);
        self.geometry = ElementGeometry::new(
            base_geom.x,
            base_geom.y,
            base_geom.width.max(size.width),
            base_geom.height + size.height + 6,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_history::flags::MessageFlags;
    use models_history::item::HistoryItem;
    use models_peers::PeerId;

    #[test]
    fn test_incoming_message_view() {
        let mut item = HistoryItem::new(101, PeerId::from_raw(1000), 1700000000);
        item.text = "Hello there!".to_string();
        let view = MessageView::new(&item, "14:30", 120, 20, 600);

        assert_eq!(view.id, 101);
        assert!(!view.is_out);
        assert_eq!(view.delivery_status, MessageDeliveryStatus::Read);
        assert_eq!(view.date_text, "14:30");
        assert!(view.geometry.height > 0);
    }

    #[test]
    fn test_outgoing_message_with_media() {
        let mut item = HistoryItem::new(102, PeerId::from_raw(1000), 1700000050);
        item.flags = MessageFlags(MessageFlags::Outgoing | MessageFlags::ClientSideUnread);
        item.text = "Photo caption".to_string();

        let mut view = MessageView::new(&item, "14:31", 100, 20, 600);

        assert!(view.is_out);
        assert_eq!(view.delivery_status, MessageDeliveryStatus::Sent);

        let initial_h = view.geometry.height;
        view.set_media_size(Size::new(320, 240), 100, 20, 600);
        assert!(view.geometry.height > initial_h + 240);
    }
}
