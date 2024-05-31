import React, {useState, useEffect, ChangeEvent, FormEvent, Component} from 'react';
import './App.css';
import {get_chat, configure} from 'nervo-wasm';

interface ChatMessage {
    role: string;
    content: string;
}

interface Chat {
    chat_id: number,
    messages:ChatMessage[]
}

// TODO: Delete
const chatId = "8";
const userId = "123456789";

function App() {

    // let conversation = [];
    const [conversation, setConversation] = useState<JSX.Element[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | boolean>(false);

    configure();

    async function fetchChat() {
        try {
            console.log("3. fetch data")
            let chatString = await get_chat(chatId, userId);
            console.log("After get chat")
            console.log("chat messages {:?}", chatString)
            let chat: Chat = JSON.parse(chatString);
            console.log("Messages parse done")

            const conversationElements = chat.messages.map((message, index) => {
                console.log("handle message!")
                if (message.role === "User") {
                    console.log("User message")
                    return <RequestContent key={index} text={message.content}/>;
                } else { //"Assistant"
                    console.log("assistant message")
                    return <ReplyContent key={index} text={message.content}/>;
                }
            });

            console.log("set conversation " + conversationElements.length)
            setConversation(conversationElements);

            // conversation = []

            // for (let i = 0; i < messages.length; i++) {
            //     let message = messages[i];
            //     if (message["role"] === "User") {
            //         conversation.push(<RequestContent text={message["content"]}/>);
            //         continue;
            //     }
            //     if (message["role"] === "Assistant") {
            //         conversation.push(<ReplyContent text={message["content"]}/>);
            //     }
            // }

        } catch (error) {
            console.error("Failed to fetch chat: ", error);
            setError("Failed to fetch chat");
        } finally {
            console.log("Finally!!!!")
            setLoading(false);
        }
    }

    console.log("#1. begin")

    useEffect(() => {
        fetchChat().catch(console.error);
    }, []);

    //const sendMessage = (messageText) => {
    //    send_message(chatId, userId, "User", messageText)
    //}

    if (loading) {
        return <div>Loading...</div>;
    }

    if (error) {
        console.log("Error!!!!", error)
        return <div>{error}</div>;
    }

    return (
        <div className="flex h-[97vh] w-full flex-col">
            {/* Prompt Messages */}
            <div
                className="flex-1 overflow-y-auto  bg-slate-300 text-sm leading-6 text-slate-900 shadow-md dark:bg-slate-800 dark:text-slate-300 sm:text-base sm:leading-7"
            >
                {conversation}
            </div>

            {/* Prompt message input */}
            <MessagingPanel/>
        </div>
    )
}

interface ReplyContentProps {
    text: string;
}

const ReplyContent: React.FC<ReplyContentProps> = ({text}) => {
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
                <LikeDislike/>
            </div>
        </div>
    );
};

class RequestContent extends Component<any, any> {
    render() {
        return (
            <div className="flex flex-row px-4 py-8 sm:px-6">
                <img
                    className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                    src="https://dummyimage.com/256x256/363536/ffffff&text=U"
                />

                <div className="flex max-w-3xl items-center">
                    <p>{this.props.text}</p>
                </div>
            </div>
        );
    }
}

const LikeDislike: React.FC = () => {
    return (
        <div className="mt-4 flex flex-row justify-start gap-x-2 text-slate-500 lg:mt-0">
            <button className="hover:text-blue-600">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 24 24"
                    strokeWidth="2"
                    stroke="currentColor"
                    fill="none"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                >
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                    <path
                        d="M7 11v8a1 1 0 0 1 -1 1h-2a1 1 0 0 1 -1 -1v-7a1 1 0 0 1 1 -1h3a4 4 0 0 0 4 -4v-1a2 2 0 0 1 4 0v5h3a2 2 0 0 1 2 2l-1 5a2 3 0 0 1 -2 2h-7a3 3 0 0 1 -3 -3"
                    ></path>
                </svg>
            </button>
            <button className="hover:text-blue-600" type="button">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 24 24"
                    strokeWidth="2"
                    stroke="currentColor"
                    fill="none"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                >
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                    <path
                        d="M7 13v-8a1 1 0 0 0 -1 -1h-2a1 1 0 0 0 -1 1v7a1 1 0 0 0 1 1h3a4 4 0 0 1 4 4v1a2 2 0 0 0 4 0v-5h3a2 2 0 0 0 2 -2l-1 -5a2 3 0 0 0 -2 -2h-7a3 3 0 0 0 -3 3"
                    ></path>
                </svg>
            </button>
            <button className="hover:text-blue-600" type="button">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 24 24"
                    strokeWidth="2"
                    stroke="currentColor"
                    fill="none"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                >
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                    <path
                        d="M8 8m0 2a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2z"
                    ></path>
                    <path
                        d="M16 8v-2a2 2 0 0 0 -2 -2h-8a2 2 0 0 0 -2 2v8a2 2 0 0 0 2 2h2"
                    ></path>
                </svg>
            </button>
        </div>
    );
};

interface MessagingPanelProps {
    //sendMessage: (messageText: string) => void;
}

const MessagingPanel: React.FC<MessagingPanelProps> = (/*{sendMessage}*/) => {
    const [messageText, setMessageText] = useState('');

    const handleMessageChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
        setMessageText(event.target.value);
    };

    const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        if (messageText.trim() !== '') {
            //sendMessage(messageText);
            setMessageText('');
        }
    };

    return (
        <form className="mt-2" onSubmit={handleSubmit}>
            <label htmlFor="chat-input" className="sr-only">Enter your prompt</label>
            <div className="relative">
                <textarea
                    id="chat-input"
                    name="chat-input"
                    rows={1}
                    className="w-full resize-none border-0 bg-transparent p-0 text-gray-900 placeholder:text-gray-400 focus:ring-0 sm:text-sm sm:leading-6"
                    placeholder="Enter your prompt..."
                    value={messageText}
                    onChange={handleMessageChange}
                />
                <button
                    type="submit"
                    className="absolute inset-y-0 right-0 flex items-center bg-blue-500 text-white px-3 py-1 rounded-lg hover:bg-blue-600 focus:outline-none"
                >
                    Send
                </button>
            </div>
        </form>
    );
};

export default App;