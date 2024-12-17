import {useState, useEffect, useRef} from 'react';
import {LlmChat, LlmMessage, LlmMessageRole, NervoClient} from "nervo-wasm";

import ReplyContent from "./components/reply-content.tsx";
import MessagingPanel from './components/messaging-panel.tsx';
import Header from './components/header.tsx';
import RequestContent from './components/request-content.tsx';

// import SlidingPanel from './components/sliding-panel.tsx';

interface AppProps {
    header: string;
    title: string;
    subtitle: string;
    height: string;
}

function App(props: AppProps) {
    const [conversation, setConversation] = useState<JSX.Element[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | boolean>(false);
    const [isTyping, setIsTyping] = useState(false);
    const [nervoClient, setNervoClient] = useState<NervoClient>();
    const messagesEndRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const serverPort: number = import.meta.env.VITE_SERVER_PORT;
        const runMode = import.meta.env.MODE;
        const agentType = import.meta.env.VITE_AGENT_TYPE;

        NervoClient.configure_tracing();

        init().catch(console.error);

        async function init() {
            const nervoClient = await initNervoClient();
            await fetchChat(nervoClient);
        }

        async function initNervoClient() {
            const nervoClient = await NervoClient.init(serverPort, runMode, agentType); // Change to 'initialization' after rebuild wasm module
            setNervoClient(nervoClient);
            return nervoClient;
        }

        async function fetchChat(nervoClient: NervoClient) {
            try {
                const chat: LlmChat = await nervoClient.get_chat();
                console.log('chatString', chat);

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
                setError(JSON.stringify(error, null, 2));
            } finally {
                setLoading(false);
            }
        }
    }, []);

    useEffect(() => {
        scrollToBottom();

        function scrollToBottom() {
            messagesEndRef.current?.scrollIntoView({behavior: "smooth"});
        }
    }, [conversation])

    if (loading || nervoClient === undefined) {
        return <div>Loading...</div>;
    }

    async function handleSendMessage(messageText: string) {
        setConversation(prevConversation => [
            ...prevConversation,
            <RequestContent key={prevConversation.length} text={messageText}/>
        ]);
        setIsTyping(true); // <-- show "typing..." message

        try {
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-expect-error
            const responseMessage = await nervoClient.send_message(messageText);
            console.log(` responseString ${JSON.stringify(responseMessage)}`)

            if (responseMessage.meta_info.role === LlmMessageRole.Assistant) {
                const msg = responseMessage.content.text();
                setConversation(prevConversation => [
                    ...prevConversation,
                    <ReplyContent key={prevConversation.length} text={msg}/>
                ]);
            }
        } catch (error) {
            console.error(" Failed to send message: ", error);
            setError("Failed to send message");
        } finally {
            setIsTyping(false); // <-- hide "typing..." message
        }
    }

    if (error) {
        console.log(error);
        //console.log("Error!!!!", error);
        return <div>{error}</div>;
    }

    const chatClassName = "flex w-full flex-col";

    return (
        <div className={chatClassName} style={{height: props.height}}>
            <Header header={props.header} title={props.title} subtitle={props.subtitle}/>

            <div
                className="flex-1 overflow-y-auto bg-slate-300 text-sm leading-6 text-slate-900 shadow-md dark:bg-[#30333d] dark:text-slate-300 sm:text-base sm:leading-7"
            >
                {conversation}
                {isTyping && (
                    <div className="p-4 text-sm text-gray-500">Typing...</div>
                )}
                <div ref={messagesEndRef} />
            </div>

            {/*<SlidingPanel buttons={['Option A', 'Option B', 'Option C']}/>*/}

            <MessagingPanel sendMessage={handleSendMessage}/>
        </div>
    );
}

export default App;
