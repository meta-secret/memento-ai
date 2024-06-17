import React, { useState, useEffect, useRef, ChangeEvent, FormEvent, Component } from 'react';
import './App.css';
import { ApiUrl, NervoClient } from "nervo-wasm";
import Cookies from 'js-cookie';

interface ChatMessage {
    role: string;
    content: string;
}

interface Chat {
    chat_id: number;
    messages: ChatMessage[];
}

function App() {
    const [conversation, setConversation] = useState<JSX.Element[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | boolean>(false);
    const messagesEndRef = useRef<HTMLDivElement | null>(null);
    const userId = getUserId();
    const chatId = getChatId();

    console.log("Running mode:" + import.meta.env.MODE);
    let apiUrl = ApiUrl.prod();
    if (import.meta.env.DEV) {
        let serverPort: number = import.meta.env.VITE_SERVER_PORT;
        console.log("port: " + serverPort);
        apiUrl = ApiUrl.dev(serverPort);
    }

    const nervoClient = NervoClient.new(apiUrl);
    nervoClient.configure();

    useEffect(() => {
        fetchChat().catch(console.error);
    }, []);

    useEffect(() => {
        scrollToBottom();
    }, [conversation]);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
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
        let chatId = Cookies.get('chatId');
        if (!chatId) {
            chatId = Math.floor(Math.random() * 0xFFFFFFFF).toString();
            Cookies.set('chatId', chatId);
        }
        console.log("chatId from cookies:");
        return chatId;
    }

    async function fetchChat() {
        try {
            let chatString = await nervoClient.get_chat(chatId);
            let chat: Chat = JSON.parse(chatString);

            const conversationElements = chat.messages.map((message, index) => {
                if (message.role === 'user') {
                    return <RequestContent key={index} text={message.content} role={message.role} />;
                } else {
                    return <ReplyContent key={index} text={message.content} role={message.role} />;
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
            <RequestContent key={prevConversation.length} text={messageText} role="user" />
        ]);

        try {
            const numUserId = parseInt(userId, 10);
            let resultUserId = numUserId >>> 0;
            let responseString = await nervoClient.send_message(chatId, resultUserId, "user", messageText);
            let responseMessage: ChatMessage = JSON.parse(responseString);

            setConversation(prevConversation => [
                ...prevConversation,
                <ReplyContent key={prevConversation.length + 1} text={responseMessage.content} role={responseMessage.role} />
            ]);
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
            <div
                className="flex-1 overflow-y-auto bg-slate-300 text-sm leading-6 text-slate-900 shadow-md dark:bg-slate-800 dark:text-slate-300 sm:text-base sm:leading-7"
            >
                {conversation}
                <div ref={messagesEndRef} />
            </div>

            <MessagingPanel sendMessage={handleSendMessage} />
        </div>
    );
}

interface ReplyContentProps {
    text: string;
    role: string;
}

const ReplyContent: React.FC<ReplyContentProps> = ({ text, role }) => {
    console.log(`ROLE: ${role}`)
    return (
        <div className="flex bg-slate-100 px-4 py-8 dark:bg-slate-900 sm:px-6">
            <img
                className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                src="https://dummyimage.com/256x256/354ea1/ffffff&text=G"
                alt="Assistant Avatar"
            />
            <div className="flex w-full flex-col items-start lg:flex-row lg:justify-between">
                <p className="max-w-3xl">
                    {text}
                </p>
            </div>
        </div>
    );
};

interface RequestContentProps {
    text?: string;
    role: string;
}

class RequestContent extends Component<RequestContentProps, any> {
    render() {
        return (
            <div className="flex flex-row px-4 py-8 sm:px-6">
                <img
                    className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                    src="https://dummyimage.com/256x256/363536/ffffff&text=U"
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

const MessagingPanel: React.FC<MessagingPanelProps> = ({ sendMessage }) => {
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
        <form className="mt-2" onSubmit={handleSubmit}>
            <label htmlFor="chat-input" className="sr-only">Enter your prompt</label>
            <div className="relative">
                <button
                    type="button"
                    className="absolute inset-y-0 left-0 flex items-center pl-3 text-slate-500 hover:text-blue-500 dark:text-slate-400 dark:hover:text-blue-500"
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
                    className="block w-full resize-none rounded-xl border-none bg-slate-200 p-4 pl-16 pr-20 text-sm text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-slate-800 dark:text-slate-200 dark:placeholder-slate-400 dark:focus:ring-blue-500 sm:text-base"
                    placeholder="Enter your prompt"
                    value={messageText}
                    onChange={handleMessageChange}
                />
                <button
                    type="submit"
                    className="absolute bottom-2 right-2.5 rounded-lg bg-blue-700 px-4 py-2 text-sm font-medium text-slate-200 hover:bg-blue-800 focus:outline-none focus:ring-4 focus:ring-blue-300 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800 sm:text-base"
                >
                    Send <span className="sr-only">Send message</span>
                </button>
            </div>
        </form>
    );
};

export default App;
