use gpui::{App, AppContext, Entity, Global, Window};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::{
    app::{components::node_renderer::NodeRenderer, states::repository_state::RepositoryState},
    domain::database::document::DocumentModel,
};

#[derive(Clone)]
pub struct PartialDocument {
    pub uid: i32,
    pub title: String,
}

#[derive(Clone)]
pub struct Document {
    pub uid: i32,
    pub title: String,
    pub nodes: Vec<Value>,
    pub renderer: Option<Entity<NodeRenderer>>,
}

#[derive(Clone, PartialEq)]
pub enum PersistenceState {
    Pending,
    Idle,
}

pub struct DocumentState {
    pub opened_document_ids: Vec<i32>,
    pub documents: Vec<Document>,

    pub current_opened_document: Option<i32>,

    pub persistence: PersistenceState,
    pub last_change: Option<Instant>,
    pub pending_notification: bool,
}

impl DocumentState {
    pub fn get_current_document_index(&self) -> Option<usize> {
        self.documents.iter().position(|doc| {
            doc.uid
                == self
                    .current_opened_document
                    .as_ref()
                    .map(|doc| doc.clone())
                    .unwrap_or_default()
        })
    }

    pub fn get_previous_document(&self, uid: i32) -> Option<Document> {
        let current_index = self.documents.iter().position(|doc| doc.uid == uid);
        if let Some(index) = current_index {
            if index > 0 {
                Some(self.documents[index - 1].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add_document(&mut self, id: i32) {
        if self.documents.iter().any(|doc| doc.uid == id) {
            self.opened_document_ids.push(id);
            return;
        }
        // let already_has_document = self.documents.iter().any(|element| element.uid == uid);
        // if !already_has_document {
        //     let renderer = NodeRenderer::new(nodes.clone(), window, cx);
        //     self.documents.push(Document {
        //         uid,
        //         title,
        //         nodes,
        //         renderer: Some(cx.new(|_| renderer)),
        //     });
        // }
    }

    pub fn add_document_and_focus(&mut self, id: i32) {
        self.add_document(id);
        self.current_opened_document = Some(id);

        // self.current_document = self
        //     .documents
        //     .clone()
        //     .into_iter()
        //     .find(|element| element.uid == uid)
    }

    pub fn add_persisted_document(&mut self, uid: i32, title: String, nodes: Vec<Value>) {
        let already_has_document = self.documents.iter().any(|element| element.uid == uid);
        if !already_has_document {
            self.documents.push(Document {
                uid,
                title,
                nodes,
                renderer: None,
            });
        }
    }

    pub fn ensure_renderer_for(&mut self, uid: i32, window: &mut Window, cx: &mut App) {
        if let Some(doc) = self.documents.iter_mut().find(|d| d.uid == uid) {
            if doc.renderer.is_none() {
                let renderer = NodeRenderer::new(doc.nodes.clone(), window, cx);
                doc.renderer = Some(cx.new(|_| renderer));
            }
        }
    }

    pub fn remove_document(&mut self, uid: i32) {
        self.documents.retain(|element| element.uid != uid);
    }

    pub fn mark_changed(&mut self, _: &mut Window, cx: &mut App) {
        let trigger_time = Instant::now();

        self.persistence = PersistenceState::Pending;
        self.last_change = Some(trigger_time);

        let documents = cx.global::<RepositoryState>().documents.clone();

        let document = self
            .documents
            .clone()
            .into_iter()
            .find(|document| Some(document.uid) == self.current_opened_document);

        if let Some(document) = document {
            let renderer = document.renderer;

            cx.spawn(async move |cx| {
                sleep(Duration::from_secs(1)).await;

                let _ = cx.update_global::<DocumentState, _>(move |state, cx| {
                    if let Some(last) = state.last_change {
                        if last <= trigger_time {
                            state.persistence = PersistenceState::Idle;
                            state.pending_notification = true;

                            if let Some(renderer) = renderer {
                                let nodes = {
                                    let nodes = renderer.read(cx).state.clone();
                                    let nodes = nodes.read(cx).get_nodes().clone();
                                    nodes
                                        .iter()
                                        .map(|node| node.element.get_data(cx))
                                        .collect::<Vec<_>>()
                                };

                                let document_model = DocumentModel {
                                    id: document.uid.clone(),
                                    title: document.title.clone(),
                                    content: Value::from_iter(nodes),
                                };

                                let _ = cx
                                    .spawn(async move |_| {
                                        documents.update_document(document_model).await
                                    })
                                    .detach();
                            }
                        }
                    } else {
                        state.persistence = PersistenceState::Idle;
                        state.pending_notification = true;
                    }
                });

                Ok::<_, anyhow::Error>(())
            })
            .detach();
        }
    }
}

impl Default for DocumentState {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            opened_document_ids: Vec::new(),
            current_opened_document: None,
            persistence: PersistenceState::Idle,
            last_change: None,
            pending_notification: false,
        }
    }
}

impl Global for DocumentState {}
