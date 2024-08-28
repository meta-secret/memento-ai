import {useState, useEffect, useRef} from 'react';
import {
    ApiUrl,
    ClientRunModeUtil,
    LlmChat,
    LlmMessage,
    LlmMessageRole,
    NervoAgentType,
    NervoClient,
    WasmIdGenerator
} from "nervo-wasm";

import Cookies from 'js-cookie';
import ReplyContent from "./components/reply-content.tsx";
import MessagingPanel from './components/messaging-panel.tsx';
import Header from './components/header.tsx';
import RequestContent from './components/request-content.tsx';
// import SlidingPanel from './components/sliding-panel.tsx';

interface AppProps {
    header: string;
    title: string;
    subtitle: string;
}

function App(props: AppProps) {
    const [conversation, setConversation] = useState<JSX.Element[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | boolean>(false);
    const [isTyping, setIsTyping] = useState(false);
    const messagesEndRef = useRef<HTMLDivElement | null>(null);
    const userId = getUserId();
    const chatId = getChatId();

    const serverPort: number = import.meta.env.VITE_SERVER_PORT;
    const mode = ClientRunModeUtil.parse(import.meta.env.MODE);
    const apiUrl = ApiUrl.get(serverPort, mode);
    //Need to use it to send to server
    const agentType = NervoAgentType.try_from(import.meta.env.VITE_AGENT_TYPE);

    console.log("Agent type:", NervoAgentType.get_name(agentType), ", port: " + serverPort);

    const nervoClient = NervoClient.new(apiUrl, agentType);

    useEffect(() => {
        nervoClient.configure();
    }, []);

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
            userId = WasmIdGenerator.generate_uuid();
            Cookies.set('userId', userId);
        }
        return userId;
    }

    function getChatId(): bigint {
        let chatId: bigint = BigInt(Cookies.get('chatId')!);
        if (!chatId) {
            chatId = WasmIdGenerator.generate_u64();
            Cookies.set('chatId', chatId.toString());
        }
        return chatId;
    }

    async function fetchChat() {
        try {
            const chat: LlmChat = await nervoClient.get_chat(BigInt(chatId));
            console.log(`WEB: chatString ${JSON.stringify(chat)}`)

            const conversationElements = chat.messages.map((message: LlmMessage, index: number) => {
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
        setIsTyping(true); // <-- show "typing..." message

        try {
            const responseMessage: LlmMessage = await nervoClient.send_message(BigInt(chatId), BigInt(userId), messageText);
            console.log(`WEB: responseString ${JSON.stringify(responseMessage)}`)

            if (responseMessage.meta_info.role === LlmMessageRole.Assistant) {
                const msg = responseMessage.content.text();
                setConversation(prevConversation => [
                    ...prevConversation,
                    <ReplyContent key={prevConversation.length} text={msg}/>
                ]);
            }
        } catch (error) {
            console.error("WEB: Failed to send message: ", error);
            setError("Failed to send message");
        } finally {
            setIsTyping(false); // <-- hide "typing..." message
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
                {isTyping && (
                    <div className="p-4 text-sm text-gray-500">Typing...</div>
                )}
                <div ref={messagesEndRef}/>
            </div>

            {/*<SlidingPanel buttons={['Option A', 'Option B', 'Option C']}/>*/}

            <MessagingPanel sendMessage={handleSendMessage}/>
        </div>
    );
}

export default App;
