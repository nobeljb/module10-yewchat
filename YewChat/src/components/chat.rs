use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{services::{event_bus::EventBus, websocket::WebsocketService}, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    current_user: String,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("Context to be set");

        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        log::debug!("Create function");

        if let Ok(_) = wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()) {
            log::debug!("Message sent successfully!");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            current_user: username,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://api.dicebear.com/7.x/adventurer-neutral/svg?seed={}",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData = serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self.wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()) {
                        log::debug!("Error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen bg-gray-900 text-white">
                <div class="flex-none w-56 h-screen bg-gray-800 border-r border-gray-700">
                    <div class="text-xl p-3 text-white font-semibold border-b border-gray-700">{"👥 Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            let is_current_user = u.name == self.current_user;
                            let user_bg_class = if is_current_user {
                                "bg-blue-600 border-blue-500"
                            } else {
                                "bg-gray-700 border-gray-600 hover:bg-gray-600"
                            };

                            html!{
                                <div class={format!("flex m-3 rounded-lg p-2 border transition-colors duration-200 {}", user_bg_class)}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full border-2 border-gray-500" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-sm justify-between">
                                            <div class="font-medium">
                                                {u.name.clone()}
                                                if is_current_user {
                                                    <span class="ml-2 text-xs bg-blue-800 px-2 py-1 rounded-full">{"You"}</span>
                                                }
                                            </div>
                                        </div>
                                        <div class="text-xs text-gray-300">
                                            if is_current_user {
                                                {"That's you!"}
                                            } else {
                                                {"Online"}
                                            }
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col bg-gray-900">
                    <div class="w-full h-14 border-b border-gray-700 bg-gray-800">
                        <div class="text-xl p-3 text-white font-semibold">{"💬 Chat Room"}</div>
                    </div>
                    <div class="w-full grow overflow-auto border-b border-gray-700 p-4 bg-gray-900">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let is_current_user = m.from == self.current_user;

                                if let Some(user) = user {
                                    let message_classes = if is_current_user {
                                        "flex items-end justify-end w-full mb-4"
                                    } else {
                                        "flex items-end w-full mb-4"
                                    };

                                    let bubble_classes = if is_current_user {
                                        "bg-blue-600 text-white rounded-tl-lg rounded-tr-lg rounded-bl-lg border border-blue-500 max-w-xs lg:max-w-md"
                                    } else {
                                        "bg-gray-700 text-white rounded-tl-lg rounded-tr-lg rounded-br-lg border border-gray-600 max-w-xs lg:max-w-md"
                                    };

                                    html!{
                                        <div class={message_classes}>
                                            if !is_current_user {
                                                <img class="w-8 h-8 rounded-full mr-3 border-2 border-gray-600" src={user.avatar.clone()} alt="avatar"/>
                                            }
                                            <div class={bubble_classes}>
                                                <div class="p-3">
                                                    <div class="text-sm font-medium mb-1">
                                                        if is_current_user {
                                                            {"You"}
                                                        } else {
                                                            {m.from.clone()}
                                                        }
                                                    </div>
                                                    <div class="text-sm">
                                                        if m.message.ends_with(".gif") {
                                                            <img class="mt-2 rounded max-w-full" src={m.message.clone()}/>
                                                        } else {
                                                            {m.message.clone()}
                                                        }
                                                    </div>
                                                </div>
                                            </div>
                                            if is_current_user {
                                                <img class="w-8 h-8 rounded-full ml-3 border-2 border-blue-500" src={user.avatar.clone()} alt="avatar"/>
                                            }
                                        </div>
                                    }
                                } else {
                                    html!{}
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-16 flex px-4 items-center bg-gray-800 border-t border-gray-700">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Type your message..."
                            class="block w-full py-3 pl-4 mx-3 bg-gray-700 border border-gray-600 rounded-full outline-none focus:border-blue-500 focus:bg-gray-600 text-white placeholder-gray-400 transition-colors duration-200"
                            name="message"
                            required=true
                        />
                        <button
                            onclick={submit}
                            class="p-3 shadow-lg bg-blue-600 hover:bg-blue-700 w-12 h-12 rounded-full flex justify-center items-center transition-colors duration-200 border border-blue-500"
                        >
                            <svg fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-white">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}