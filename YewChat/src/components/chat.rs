use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use crate::services::event_bus::EventBus;
use crate::{User, services::websocket::WebsocketService};

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
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }
    
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
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
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        
        html! {
            <div class="w-full h-screen flex bg-gradient-to-br from-blue-50 via-indigo-50 to-slate-50 overflow-hidden">

                <div class="hidden md:flex md:flex-none md:w-64 lg:w-72 h-full bg-white/95 backdrop-blur-xl border-r border-blue-200/70 shadow-xl flex-col">
                    <div class="flex-none text-xl md:text-2xl p-4 md:p-5 font-bold border-b border-blue-200/50 text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-indigo-600">
                        <div class="flex items-center space-x-2">
                            <div class="w-2 h-2 bg-emerald-400 rounded-full animate-pulse shadow-sm"></div>
                            {"Users Online"}
                        </div>
                    </div>
                    <div class="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-blue-300 scrollbar-track-transparent">
                        {
                            self.users.clone().iter().enumerate().map(|(index, u)| {
                                let animation_delay = format!("animation-delay: {}ms", index * 100);
                                
                                html! {
                                    <div class="animate-fade-in-up" style={animation_delay}>
                                        <div class="flex m-2 md:m-3 bg-gradient-to-r from-blue-50/70 to-indigo-50/70 hover:from-blue-100/80 hover:to-indigo-100/80 rounded-xl p-3 md:p-4 items-center cursor-pointer hover:shadow-lg hover:scale-[1.02] transition-all duration-300 border border-blue-100/60 hover:border-blue-200">
                                            <div class="relative">
                                                <img class="w-12 h-12 md:w-14 md:h-14 lg:w-16 lg:h-16 rounded-full object-cover border-2 border-blue-300 shadow-md" 
                                                    src={u.avatar.clone()} alt="avatar" />
                                                <div class="absolute -bottom-1 -right-1 w-4 h-4 bg-emerald-400 border-2 border-white rounded-full animate-pulse shadow-sm"></div>
                                            </div>
                                            <div class="flex-1 ml-3 md:ml-4 min-w-0">
                                                <div class="flex text-sm md:text-base justify-between items-center font-semibold text-transparent bg-clip-text bg-gradient-to-r from-blue-700 to-indigo-700">
                                                    <div class="truncate pr-2">{u.name.clone()}</div>
                                                    <div class="text-xs text-emerald-600 font-medium flex items-center">
                                                        <div class="w-2 h-2 bg-emerald-400 rounded-full mr-1 animate-pulse"></div>
                                                        {"Online"}
                                                    </div>
                                                </div>
                                                <div class="text-xs text-blue-500 truncate mt-1 font-medium">
                                                    {"Available to chat"}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                <div class="flex-1 h-full flex flex-col min-w-0">

                    <div class="flex-none w-full h-16 md:h-18 border-b border-blue-200/60 flex items-center px-4 md:px-6 bg-white/95 backdrop-blur-xl shadow-sm">
                        <div class="flex items-center space-x-3">
                            <div class="text-2xl md:text-3xl">{"💬"}</div>
                            <div class="text-xl md:text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-indigo-600">
                                {"Chat Hub"}
                            </div>
                            <div class="hidden md:flex items-center text-sm text-blue-600 ml-4 bg-blue-50 px-3 py-1 rounded-full">
                                <div class="w-2 h-2 bg-emerald-400 rounded-full mr-2 animate-pulse"></div>
                                {format!("{} online", self.users.len())}
                            </div>
                        </div>

                        <button class="md:hidden ml-auto p-3 text-blue-600 hover:bg-blue-50 rounded-full transition-colors duration-200">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                            </svg>
                        </button>
                    </div>

                    <div class="flex-1 w-full overflow-y-auto p-4 md:p-6 lg:p-8 space-y-4 md:space-y-6 bg-gradient-to-br from-blue-50/30 via-indigo-50/30 to-slate-50/50 scrollbar-thin scrollbar-thumb-blue-300 scrollbar-track-transparent">
                        {
                            self.messages.iter().enumerate().map(|(index, m)| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                let is_even = index % 2 == 0;
                                let animation_delay = format!("animation-delay: {}ms", index * 150);
                                
                                html! {
                                    <div class="animate-fade-in-left" style={animation_delay}>
                                        <div class={format!("flex items-end max-w-full sm:max-w-[85%] md:max-w-[75%] lg:max-w-[65%] {} rounded-2xl shadow-sm hover:shadow-md transition-all duration-300 border backdrop-blur-sm", 
                                            if is_even { "bg-white/90 border-blue-200/50 hover:bg-white/95" } else { "bg-gradient-to-r from-blue-50/80 to-indigo-50/80 border-indigo-200/50 hover:from-blue-100/90 hover:to-indigo-100/90" })}>
                                            <div class="flex-shrink-0">
                                                <div class="relative">
                                                    <img class="w-10 h-10 md:w-12 md:h-12 rounded-full m-3 md:m-4 object-cover border-2 border-blue-200 shadow-sm" 
                                                        src={user.avatar.clone()} alt="avatar" />
                                                    <div class="absolute bottom-2 right-2 w-3 h-3 bg-emerald-400 border-2 border-white rounded-full"></div>
                                                </div>
                                            </div>
                                            <div class="flex-1 p-4 md:p-5 min-w-0">
                                                <div class="flex items-center space-x-2 mb-2">
                                                    <div class="text-sm md:text-base font-semibold text-transparent bg-clip-text bg-gradient-to-r from-blue-700 to-indigo-700">
                                                        {m.from.clone()}
                                                    </div>
                                                    <div class="text-xs text-blue-400 font-medium">
                                                        {"• just now"}
                                                    </div>
                                                </div>
                                                <div class="text-sm md:text-base text-slate-700 leading-relaxed">
                                                    {
                                                        if m.message.ends_with(".gif") {
                                                            html! {
                                                                <img class="rounded-xl shadow-md max-w-full h-auto hover:scale-105 transition-transform duration-300" src={m.message.clone()} alt="gif" />
                                                            }
                                                        } else {
                                                            html! {
                                                                <div class="break-words">{m.message.clone()}</div>
                                                            }
                                                        }
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                
                    <div class="flex-none w-full h-16 md:h-18 flex px-4 md:px-6 lg:px-8 py-3 md:py-4 items-center bg-white/95 backdrop-blur-xl border-t border-blue-200/60 shadow-sm">
                        <div class="flex-1 flex items-center space-x-3 md:space-x-4">
                            <div class="relative flex-1">
                                <input 
                                    ref={self.chat_input.clone()} 
                                    type="text" 
                                    placeholder="Type your message..." 
                                    class="w-full py-3 md:py-4 px-5 md:px-6 bg-gradient-to-r from-slate-50 to-blue-50 hover:from-white hover:to-blue-50 rounded-full outline-none text-sm md:text-base text-slate-800 placeholder-blue-400 focus:ring-2 focus:ring-blue-400 focus:bg-white transition-all duration-300 border border-blue-200/50 focus:border-blue-300 shadow-sm hover:shadow-md" 
                                    name="message" 
                                    required=true 
                                />
                            </div>
                            <button 
                                onclick={submit} 
                                type="submit"
                                class="flex-shrink-0 w-12 h-12 md:w-14 md:h-14 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 rounded-full flex justify-center items-center text-white shadow-md hover:shadow-lg transition-all duration-300 focus:outline-none focus:ring-4 focus:ring-blue-300/50 transform hover:scale-105 active:scale-95"
                                aria-label="Send message"
                            >
                                <svg 
                                    fill="none" 
                                    stroke="currentColor" 
                                    stroke-width="2.5" 
                                    viewBox="0 0 24 24" 
                                    xmlns="http://www.w3.org/2000/svg" 
                                    class="w-5 h-5 md:w-6 md:h-6 transform rotate-45 transition-transform duration-200"
                                >
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}