use ratatui::crossterm::event::Event as CrosstermEvent;

#[derive(Debug, Clone)]
struct NewPlotEvent {
    pub topic: String,
    pub field: Vec<usize>,
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    None,
    Key(CrosstermEvent),
    NewPlot(NewPlotEvent),
}
