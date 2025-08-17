struct Selectable<T> {
    items: Vec<T>,
    selected_index: usize,
}

impl<T> Selectable<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            selected_index: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() && self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected_index)
    }

    pub fn remove_selected(&mut self) {
        if !self.items.is_empty() {
            self.items.remove(self.selected_index);
            self.selected_index = self.items.len().saturating_sub(1);
        }
    }
}

mod test {
    use super::*;

    #[test]
    fn test_selectable() {
        let mut selectable = Selectable::new(vec![1, 2, 3]);
        assert_eq!(selectable.selected_item(), Some(&1));

        selectable.next();
        assert_eq!(selectable.selected_item(), Some(&2));

        selectable.next();
        assert_eq!(selectable.selected_item(), Some(&3));

        selectable.next();
        assert_eq!(selectable.selected_item(), Some(&3)); // Should stay at last item

        selectable.previous();
        assert_eq!(selectable.selected_item(), Some(&2));

        selectable.previous();
        assert_eq!(selectable.selected_item(), Some(&1));

        selectable.previous();
        assert_eq!(selectable.selected_item(), Some(&1)); // Should stay at first item
    }
}
