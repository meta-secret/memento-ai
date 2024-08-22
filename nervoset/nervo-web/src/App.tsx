import React, {useState, useEffect, useRef, ChangeEvent, FormEvent, Component} from 'react';
import {ApiUrl, LlmChat, LlmMessage, LlmMessageRole, NervoAgentType, NervoClient} from "nervo-wasm";
import Cookies from 'js-cookie';
import ReplyContent from "./components/reply-content.tsx";

interface AppProps {
    header: string;
    title: string;
    subtitle: string;
}
function App(props: AppProps) {
    const [conversation, setConversation] = useState<JSX.Element[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | boolean>(false);
    const messagesEndRef = useRef<HTMLDivElement | null>(null);
    const userId = getUserId();
    const chatId = getChatId();

    let apiUrl = ApiUrl.prod();
    let serverPort: number = import.meta.env.VITE_SERVER_PORT;
    console.log("port: " + serverPort);
    if (import.meta.env.MODE === "localDev") {
        apiUrl = ApiUrl.local(serverPort);
    }

    if (import.meta.env.MODE === "dev") {
        apiUrl = ApiUrl.dev(serverPort);
    }

    if (import.meta.env.MODE === "prod") {
        apiUrl = ApiUrl.prod();
    }

    //Need to use it to send to server
    const agentType = NervoAgentType.try_from(import.meta.env.VITE_AGENT_TYPE);
    console.log("Agent type: ", agentType)

    const nervoClient = NervoClient.new(apiUrl, agentType);
    nervoClient.configure();

    useEffect(() => {
        fetchChat().catch(console.error);
    }, []);

    useEffect(() => {
        scrollToBottom();
    }, [conversation]);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({behavior: "smooth"});
    };

    function getUserId() {
        let userId = Cookies.get('userId');
        if (!userId) {
            userId = Math.floor(Math.random() * 0xFFFFFFFF).toString();
            Cookies.set('userId', userId);
        }
        return userId;
    }

    function getChatId() {
        let chatId: number = Number(Cookies.get('chatId'));
        if (!chatId) {
            chatId = Math.floor(Math.random() * 0xFFFFFFFF);
            Cookies.set('chatId', chatId.toString());
        }
        return chatId;
    }

    async function fetchChat() {
        try {
            let chat: LlmChat = await nervoClient.get_chat(BigInt(chatId));
            console.log(`WEB: chatString ${JSON.stringify(chat)}`)

            const conversationElements = chat.messages.map((message, index) => {
                if (message.meta_info.role === LlmMessageRole.User) {
                    return <RequestContent key={index} text={message.content.text()}/>;
                } else if (message.meta_info.role === LlmMessageRole.Assistant) {
                    return <ReplyContent key={index} text={message.content.text()}/>;
                } else {
                    return <ReplyContent key={index} text=""/>;
                }
            });

            setConversation(conversationElements);

        } catch (error) {
            console.error("WEB: Failed to fetch chat: ", error);
            setError("WEB: Failed to fetch chat");
        } finally {
            setLoading(false);
        }
    }

    const handleSendMessage = async (messageText: string) => {
        setConversation(prevConversation => [
            ...prevConversation,
            <RequestContent key={prevConversation.length} text={messageText}/>
        ]);

        try {
            let responseMessage: LlmMessage = await nervoClient.send_message(BigInt(chatId), BigInt(userId), messageText);
            console.log(`WEB: responseString ${JSON.stringify(responseMessage)}`)

            if (responseMessage.meta_info.role === LlmMessageRole.Assistant) {
                let msg = responseMessage.content.text();
                setConversation(prevConversation => [
                    ...prevConversation,
                    <ReplyContent key={prevConversation.length} text={msg}/>
                ]);
            }
        } catch (error) {
            console.error("WEB: Failed to send message: ", error);
            setError("Failed to send message");
        }
    };

    if (loading) {
        return <div>Loading...</div>;
    }

    if (error) {
        console.log("Error!!!!", error);
        return <div>{error}</div>;
    }

    return (
        <div className="flex h-[97vh] w-full flex-col">
            <Header header={props.header} title={props.title} subtitle={props.subtitle}/>
            <div
                className="flex-1 overflow-y-auto bg-slate-300 text-sm leading-6 text-slate-900 shadow-md dark:bg-[#30333d] dark:text-slate-300 sm:text-base sm:leading-7"
            >
                {conversation}
                <div ref={messagesEndRef}/>
            </div>

            <MessagingPanel sendMessage={handleSendMessage}/>
        </div>
    );
}

interface RequestContentProps {
    text?: string;
}

class RequestContent extends Component<RequestContentProps, any> {
    render() {
        return (
            <div className="flex flex-row px-4 py-8 sm:px-6">
                <img
                    className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                    src="https://dummyimage.com/256x256/202228/ffffff&text=U"
                    alt="User Avatar"
                />
                <div className="flex max-w-3xl items-center">
                    <p>{this.props.text}</p>
                </div>
            </div>
        );
    }
}

interface MessagingPanelProps {
    sendMessage: (messageText: string) => void;
}

const MessagingPanel: React.FC<MessagingPanelProps> = ({sendMessage}) => {
    const [messageText, setMessageText] = useState('');

    const handleMessageChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
        setMessageText(event.target.value);
    };

    const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (messageText.trim() !== '') {
            sendMessage(messageText);
            setMessageText('');
        }
    };

    return (
        <form onSubmit={handleSubmit}>
            <label htmlFor="chat-input" className="sr-only">Enter your prompt</label>
            <div className="relative">
                <button
                    type="button"
                    className="absolute left-0 rounded-lg inset-y-0 flex items-center text-slate-500 hover:text-blue-500 dark:text-slate-400 dark:hover:text-blue-500"
                >
                    <svg
                        aria-hidden="true"
                        className="h-5 w-5"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                        strokeWidth="2"
                        stroke="currentColor"
                        fill="none"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    >
                        <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                        <path
                            d="M9 2m0 3a3 3 0 0 1 3 -3h0a3 3 0 0 1 3 3v5a3 3 0 0 1 -3 3h0a3 3 0 0 1 -3 -3z"
                        ></path>
                        <path d="M5 10a7 7 0 0 0 14 0"></path>
                        <path d="M8 21l8 0"></path>
                        <path d="M12 17l0 4"></path>
                    </svg>
                    <span className="sr-only">Use voice input</span>
                </button>
                <textarea
                    id="chat-input"
                    name="chat-input"
                    rows={1}
                    className="block w-full resize-none border-none bg-slate-200 p-4 pl-20 pr-18 text-sm text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-[#202228] dark:text-slate-200 dark:placeholder-slate-400 dark:focus:ring-blue-500 sm:text-base"
                    placeholder="Ask me anything"
                    value={messageText}
                    onChange={handleMessageChange}
                />
                <button
                    type="submit"
                    className="absolute bottom-2 right-2.5 rounded-lg bg-[#929bb7] px-4 py-2 text-sm font-medium text-slate-200 hover:bg-[#7e87a2] focus:outline-none focus:ring-4 focus:ring-[#6a7390] dark:bg-[#929bb7] dark:hover:bg-[#7e87a2] dark:focus:ring-[#6a7390] sm:text-base"
                >
                    Send <span className="sr-only">Send message</span>
                </button>
            </div>
        </form>
    );
};

interface HeaderProps {
    header: string;
    title: string;
    subtitle: string;
}
const Header: React.FC<HeaderProps> = ({header, title, subtitle}) => {
    return (
        <div className="flex w-full flex-col justify-between rounded-1xl bg-slate-50 p-8 text-slate-900 ring-1 ring-slate-300 dark:bg-slate-900 dark:text-slate-200 dark:ring-slate-300/20 xl:p-10">
            <div>
                <div className="flex items-center justify-between gap-x-4">
                    <h5 id="tier-starter" className="text-sm font-semibold leading-1">{header}</h5>
                </div>
                <p className="mt-2 flex items-baseline gap-x-1">
                    <span className="text-2xl font-bold tracking-tight">{title}</span>
                </p>
                <p className="mt-2 flex items-baseline gap-x-1">
                    <span
                        className="text-sm font-semibold leading-1 text-slate-700 dark:text-slate-400">{subtitle}</span>
                </p>
            </div>
        </div>
    );
};

export default App;