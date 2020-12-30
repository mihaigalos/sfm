use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    app::{
        actions::{AppAction, FileManagerActions, PanelSide},
        state::{AppState, ModalType},
    },
    core::{
        events::Event,
        store::Store,
        ui::{component::Component, component_base::ComponentBase},
    },
};

use super::{
    create_modal::{CreateModalComponent, CreateModalProps},
    panel::PanelComponent,
};

#[derive(Clone, Default)]
pub struct RootComponentState {
    focused_panel: Option<PanelSide>,
}

pub struct RootComponent {
    base: ComponentBase<(), RootComponentState>,
    left_panel: PanelComponent,
    right_panel: PanelComponent,
    create_modal: Option<CreateModalComponent>,
}

impl RootComponent {
    pub fn new() -> Self {
        RootComponent {
            base: ComponentBase::new(None, Some(RootComponentState::default())),
            left_panel: PanelComponent::empty(),
            right_panel: PanelComponent::empty(),
            create_modal: None,
        }
    }

    fn map_state(&mut self, store: &Store<AppState, FileManagerActions>) {
        let state = store.get_state();
        if state.left_panel.is_focused {
            self.base.set_state(|_current_state| RootComponentState {
                focused_panel: Some(PanelSide::Left),
            });
        } else if state.right_panel.is_focused {
            self.base.set_state(|_current_state| RootComponentState {
                focused_panel: Some(PanelSide::Right),
            });
        } else {
            self.base.set_state(|_current_state| RootComponentState {
                focused_panel: None,
            });
        }
        self.left_panel = PanelComponent::with_panel_state(
            state.left_panel,
            PanelSide::Left,
            &state.config.icons,
        );
        self.right_panel = PanelComponent::with_panel_state(
            state.right_panel,
            PanelSide::Right,
            &state.config.icons,
        );
        if let Some(modal_type) = state.modal.clone() {
            match modal_type {
                ModalType::CreateModal {
                    panel_side,
                    panel_tab,
                    panel_tab_path,
                } => {
                    if self.create_modal.is_none() {
                        self.create_modal = Some(CreateModalComponent::with_props(
                            CreateModalProps::new(panel_side, panel_tab, panel_tab_path),
                        ));
                    }
                }
                ModalType::ErrorModal(_) => {}
            };
        }
        if self.create_modal.is_some() && state.modal.is_none() {
            self.create_modal = None;
        }
    }
}

impl Component<Event, AppState, FileManagerActions> for RootComponent {
    fn on_init(&mut self, store: &Store<AppState, FileManagerActions>) {
        self.map_state(store);
    }

    fn handle_event(
        &mut self,
        event: Event,
        store: &mut Store<AppState, FileManagerActions>,
    ) -> bool {
        let state = store.get_state();
        if let Event::Keyboard(key_evt) = event {
            if state.config.keyboard_cfg.quit.is_pressed(key_evt) {
                store.dispatch(FileManagerActions::App(AppAction::Exit));
                return true;
            }

            if let Some(ref mut create_modal) = self.create_modal {
                let result = create_modal.handle_event(event, store);
                self.map_state(store);
                return result;
            }

            if state
                .config
                .keyboard_cfg
                .focus_left_panel
                .is_pressed(key_evt)
            {
                store.dispatch(FileManagerActions::App(AppAction::FocusLeft));
                self.map_state(store);
                return true;
            }

            if state
                .config
                .keyboard_cfg
                .focus_right_panel
                .is_pressed(key_evt)
            {
                store.dispatch(FileManagerActions::App(AppAction::FocusRight));
                self.map_state(store);
                return true;
            }
        }

        let mut result = self.left_panel.handle_event(event, store);
        if result == true {
            self.map_state(store);
            return result;
        }
        result = self.right_panel.handle_event(event, store);
        self.map_state(store);

        result
    }

    fn render<TBackend: Backend>(&self, frame: &mut tui::Frame<TBackend>, _area: Option<Rect>) {
        let local_state = self.base.get_state().unwrap();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.size());
        self.left_panel.render(frame, Some(layout[0]));
        self.right_panel.render(frame, Some(layout[1]));
        if let Some(ref create_modal) = self.create_modal {
            if let Some(focused_panel) = local_state.focused_panel {
                match focused_panel {
                    PanelSide::Left => create_modal.render(frame, Some(layout[0])),
                    PanelSide::Right => create_modal.render(frame, Some(layout[1])),
                };
            } else {
                create_modal.render(frame, None);
            }
        }
    }
}
