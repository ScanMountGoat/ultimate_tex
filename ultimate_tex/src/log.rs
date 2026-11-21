use gpui_kit::{
    App, Context, IntoElement, ParentElement, Window,
    base::IndexPath,
    component::{
        label::Label,
        list::{ListDelegate, ListItem, ListState},
    },
    div,
};

#[derive(Default)]
pub struct LogListDelegate {
    pub items: Vec<String>,
}

impl ListDelegate for LogListDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.items.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        self.items
            .get(ix.row)
            .map(|item| ListItem::new(ix).child(Label::new(item.clone())))
    }

    fn set_selected_index(
        &mut self,
        _ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        cx.notify();
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> impl IntoElement {
        div()
    }
}
