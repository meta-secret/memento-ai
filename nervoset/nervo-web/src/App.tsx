import React, { useState, useEffect, ChangeEvent, FormEvent } from 'react';
import './App.css';
import { ApiUrl, NervoClient } from 'nervo-wasm';

interface ChatMessage {
    role: string;
    content: string;
}

interface Chat {
    chat_id: number;
    messages: ChatMessage[];
}

const chatId = "9";
const userId = 111;

function App() {
    const [conversation, setConversation] = useState<JSX.Element[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | boolean>(false);

    const nervoClient = NervoClient.new(ApiUrl.Dev);
    nervoClient.configure();

    useEffect(() => {
        fetchChat().catch(console.error);
    }, []);

    async function fetchChat() {
        try {
            console.log("WEB: 3. Fetch data");
            let chatString = await nervoClient.get_chat(chatId);
            console.log("WEB: After get chat");
            console.log("WEB: Chat messages: ", chatString);
            let chat: Chat = JSON.parse(chatString);
            console.log("WEB: Messages parse done");

            const conversationElements = chat.messages.map((message, index) => {
                console.log("WEB: Handle message!");
                return <RequestContent key={index} text={message.content} role={message.role} />;
            });

            console.log("WEB: Set conversation, length: " + conversationElements.length);
            setConversation(conversationElements);

        } catch (error) {
            console.error("WEB: Failed to fetch chat: ", error);
            setError("WEB: Failed to fetch chat");
        } finally {
            console.log("WEB: Finally!!!!");
            setLoading(false);
        }
    }

    const handleSendMessage = async (messageText: string) => {
        try {
            console.log("WEB: Sending message:", messageText);
            let responseString = await nervoClient.send_message(chatId, userId, "user", messageText);
            console.log("WEB: Received response:", responseString);
            let responseMessage: ChatMessage = JSON.parse(responseString);

            // Update conversation
            setConversation(prevConversation => [
                ...prevConversation,
                <RequestContent key={prevConversation.length} text={messageText} role="User" />,
                <RequestContent key={prevConversation.length + 1} text={responseMessage.content} role="Assistant" />
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
        console.log("WEB: Error!!!!", error);
        return <div>{error}</div>;
    }

    return (
        <div className="flex h-[97vh] w-full flex-col">
            <div
                className="flex-1 overflow-y-auto bg-slate-300 text-sm leading-6 text-slate-900 shadow-md dark:bg-slate-800 dark:text-slate-300 sm:text-base sm:leading-7"
            >
                {conversation}
            </div>
            <MessagingPanel sendMessage={handleSendMessage} />
        </div>
    );
}

interface RequestContentProps {
    text: string;
    role: string;
}

const RequestContent: React.FC<RequestContentProps> = ({ text, role }) => {
    const isUser = role === "User";
    return (
        <div className={`flex ${isUser ? 'flex-row' : 'flex-row-reverse'} px-4 py-8 sm:px-6`}>
            <img
                className="mr-2 flex h-8 w-8 rounded-full sm:mr-4"
                src={isUser ? "https://dummyimage.com/256x256/363536/ffffff&text=U" : "https://dummyimage.com/256x256/354ea1/ffffff&text=G"}
            />
            <div className="flex max-w-3xl items-center">
                <p>{text}</p>
            </div>
        </div>
    );
};

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
                <textarea
                    id="chat-input"
                    name="chat-input"
                    rows={1}
                    className="w-full resize-none border-0 bg-transparent p-0 text-white placeholder:text-gray-400 focus:ring-0 sm:text-sm sm:leading-6"
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
