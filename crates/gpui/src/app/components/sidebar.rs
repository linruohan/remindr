use gpui::*;
use gpui_component::{
    IconName,
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};
use smol::block_on;

use crate::{
    LoadingState,
    app::{
        screens::document_screen::DocumentScreen,
        states::{
            app_state::AppState, document_state::DocumentState, repository_state::RepositoryState,
        },
    },
    domain::database::document::DocumentModel,
};

pub struct AppSidebar {
    document_state: LoadingState<Vec<DocumentModel>>,
    // documents: Option<Vec<DocumentModel>>,
    app_state: Entity<AppState>,
}

impl AppSidebar {
    pub fn new(app_state: Entity<AppState>, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let document_repository = cx.global::<RepositoryState>().documents.clone();
            cx.spawn(async move |this, cx| {
                let documents = document_repository.get_documents().await;
                if let Ok(documents) = documents {
                    let _ = this.update(cx, |state: &mut Self, _| {
                        state.document_state = LoadingState::Loaded(documents);
                    });
                }
            })
            .detach();

            Self {
                document_state: LoadingState::Loading,
                app_state,
            }
        })
    }

    fn render_documents(
        &self,
        documents: Vec<DocumentModel>,
        cx: &mut Context<Self>,
    ) -> SidebarGroup<SidebarMenu> {
        SidebarGroup::new("Documents").child(SidebarMenu::new().children(
            documents.into_iter().map(|document| {
                let document_id = document.id.clone();
                SidebarMenuItem::new(document.title.clone())
                    .icon(IconName::File)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let document = block_on(async {
                            cx.global::<RepositoryState>()
                                .documents
                                .get_document_by_id(document_id)
                                .await
                        });

                        if let Ok(document) = document {
                            cx.update_global::<DocumentState, _>(|state, _| {
                                state.add_document_and_focus(document.id);
                            });

                            this.app_state.update(cx, |app_state, cx| {
                                let document_screen = DocumentScreen::new(cx.weak_entity());
                                app_state.navigator.push(document_screen, cx);
                            });
                        }
                    }))
                    .collapsed(false)
                    .active(cx.read_global::<DocumentState, _>({
                        move |state, _| {
                            if let Some(current_document) = state.current_opened_document.clone() {
                                current_document == document_id
                            } else {
                                false
                            }
                        }
                    }))
            }),
        ))
    }
}

impl Render for AppSidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let documents = match self.document_state.clone() {
            LoadingState::Loaded(documents) => self.render_documents(documents, cx),
            _ => SidebarGroup::new("Documents"),
        };

        Sidebar::left()
            .w(Pixels::from(240.0))
            .header(SidebarHeader::new())
            .child(documents)
            .footer(SidebarFooter::new().child("Footer"))

        // let links = vec![
        //     SidebarMenuItem::new("Chercher").icon(IconName::Search),
        //     SidebarMenuItem::new("Accueil")
        //         .icon(Icon::default().path("icons/house.svg"))
        //         .on_click(cx.listener(|this, _, _, cx| {
        //             this.app_state.update(cx, |app_state, cx| {
        //                 let home_screen = HomeScreen::new(cx.weak_entity());
        //                 app_state.navigator.push(home_screen, cx);
        //             });
        //         })),
        //     SidebarMenuItem::new("Boite de réception").icon(IconName::Inbox),
        // ];

        // let documents = if let Some(documents) = &self.documents {
        //     documents
        //         .into_iter()
        //         .map(|document| {
        //             let document_id = document.id.clone();
        //             SidebarMenuItem::new(document.title.clone())
        //                 .icon(IconName::File)
        //                 .on_click(cx.listener({
        //                     move |this, _, window, cx| {
        //                         let document = block_on(async {
        //                             cx.global::<RepositoryState>()
        //                                 .documents
        //                                 .get_document_by_id(document_id)
        //                                 .await
        //                         });

        //                         match document {
        //                             Ok(document) => {
        //                                 cx.update_global::<DocumentState, _>(|state, cx| {
        //                                     state.add_document_and_focus(
        //                                         document.id.clone(),
        //                                         document.title.clone(),
        //                                         document.content.as_array().unwrap().clone(),
        //                                         window,
        //                                         cx,
        //                                     );
        //                                 });

        //                                 this.app_state.update(cx, |app_state, cx| {
        //                                     let document_screen =
        //                                         DocumentScreen::new(cx.weak_entity());
        //                                     app_state.navigator.push(document_screen, cx);
        //                                 });
        //                             }
        //                             Err(error) => {
        //                                 window.push_notification(
        //                                     Notification::new()
        //                                         .title("Update Available")
        //                                         .message(format!("One error occurred: {}", error))
        //                                         .with_type(NotificationType::Error),
        //                                     cx,
        //                                 );
        //                             }
        //                         };
        //                     }
        //                 }))
        //                 .active(cx.read_global::<DocumentState, _>({
        //                     move |state, _| {
        //                         if let Some(current_document) = state.current_document.clone() {
        //                             current_document.uid == document_id
        //                         } else {
        //                             false
        //                         }
        //                     }
        //                 }))
        //         })
        //         .collect()
        // } else {
        //     vec![]
        // };

        // Sidebar::left()
        //     .w(Pixels::from(240.0))
        //     .header(SidebarHeader::new())
        //     .child(
        //         SidebarGroup::new("Label")
        //             .child(SidebarMenu::new().children(links))
        //             .collapsed(true),
        //     )
        //     .child(
        //         SidebarGroup::new("Documents")
        //             .child(SidebarMenu::new().children(documents).collapsed(false)),
        //     )
        //     .footer(SidebarFooter::new().child("Footer"))
    }
}
